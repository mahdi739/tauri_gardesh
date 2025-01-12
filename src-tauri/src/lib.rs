#![allow(unused)]
use enum_all_variants::AllVariants;
use genai::{
  chat::{ChatMessage, ChatOptions, ChatRequest, ChatResponseFormat, JsonSpec},
  resolver::{AuthData, AuthResolver, AuthResolverFn},
  Client, ClientConfig,
};
use kalosm::language::prompt_input;
use std::{
  cmp::Ordering::Equal, collections::HashMap, env, fmt::Debug, path::PathBuf, str::FromStr,
};
use tauri_plugin_log::{Target, TargetKind};
use tokio::sync::Mutex;

use dotenv::dotenv;
use iter_tools::Itertools;
// use reqwest::{
//   header::{self, CONTENT_TYPE},
//   Client,
// };
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Instant;
use strum::{Display, EnumString};
use strum::{VariantArray, VariantNames};

#[derive(
  Debug,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
  Hash,
  Copy,
  Display,
  Clone,
  EnumString,
  VariantArray,
  VariantNames,
)]
enum PlaceType {
  #[serde(rename = "موزه")]
  #[strum(to_string = "موزه")]
  Museum,
  #[serde(rename = "مکان تاریخی")]
  #[strum(to_string = "مکان تاریخی")]
  Historical,
  #[serde(rename = "رستوران")]
  #[strum(to_string = "رستوران")]
  Restaurant,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Location {
  x: f64,
  y: f64,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PlaceInfo {
  place_type: PlaceType,
  tags: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct PromptAnalyse {
  // entry_point: Option<Location>,
  place_infos: Vec<PlaceInfo>,
  total_count: Option<u32>,
}

fn distance_haversine(loc1: &Location, loc2: &Location) -> f64 {
  let r = 6371e3; // Earth's radius in meters
  let phi1 = loc1.y.to_radians();
  let phi2 = loc2.y.to_radians();
  let delta_phi = (loc2.y - loc1.y).to_radians();
  let delta_lambda = (loc2.x - loc1.x).to_radians();

  let a =
    (delta_phi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
  let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

  r * c
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// struct FinalResponse {
//   place_name: String,
//   place_type: PlaceType,
// }
// #[derive(Debug, Serialize, Deserialize, Clone)]
// struct FinalResponseItems {
//   items: Vec<FinalResponse>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, Default)]
// struct ResponseModel {
//   names: Vec<String>,
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Place {
  title: String,
  category: String,
  r#type: PlaceType,
  region: String,
  #[serde(default)]
  neighbourhood: String,
  location: Location,
  #[serde(default)]
  tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct Model {
  tag_pool: Vec<String>,
  items: Vec<Place>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PlaceScoringItems {
  items: Vec<PlaceScoring>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PlaceScoring {
  place: Place,
  score: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PromptClassification {
  place_type: String,
  count: u32,
  prompt_slice: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PromptClassificationArray {
  items: Vec<PromptClassification>,
}

pub fn run() {
  dotenv().ok();

  tauri::Builder::default()
    // .plugin(
    //   tauri_plugin_log::Builder::new()
    //     .target(tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout))
    //     .build(),
    // )
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![greet, answer])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
const MODEL: &str = "gemini-2.0-flash-exp";

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn greet(name: String) -> Vec<Place> {
  do_the_job(name).await
}

async fn do_the_job(name: String) -> Vec<Place> {
  // use rand::{rngs::StdRng, Rng as _, SeedableRng};
  // let mut rng: StdRng = StdRng::from_entropy();

  // let random_points = (0..20000)
  //   .map(|_| {
  //     let lat = rng.gen_range(35.60..=35.80);
  //     let lng = rng.gen_range(51.20..=51.50);
  //     [lat, lng]
  //   })
  //   .collect::<Vec<_>>();
  // use rstar::RTree;
  // let tree = RTree::bulk_load(random_points);
  // let n = Instant::now();
  // let random_points = (0..10)
  //   .map(|_| {
  //     let lat = rng.gen_range(35.60..=35.80);
  //     let lng = rng.gen_range(51.20..=51.50);
  //     [lat, lng]
  //   })
  //   .for_each(|f| {
  //     let nearest_neighbors = tree.nearest_neighbor_iter(&f).collect::<Vec<_>>();

  //     println!("{nearest_neighbors:#?}");
  //   });

  // println!("Time Passed: {}", n.elapsed().as_millis());

  // Lat: 35.60 - 35.80
  // Long: 51.20 - 51.50
  let neshan_history = serde_json::from_str::<Model>(include_str!(
    "taged_items/neshan_history_results_unique_with_tags.json"
  ))
  .unwrap();
  let neshan_museum = serde_json::from_str::<Model>(include_str!(
    "taged_items/neshan_museum_results_unique_with_tags.json"
  ))
  .unwrap();
  let neshan_restaurant = serde_json::from_str::<Model>(include_str!(
    "taged_items/neshan_restaurant_results_unique_with_tags.json"
  ))
  .unwrap();

  let system_prompt = format!("
    درخواست کاربر را تجزیه تحلیل کن.
    هر نوع مکان ذکر شده را شناسایی کن که یکی از این سه نوع است: موزه، رستوران، مکان تاریخی.
    سپس برای هر مکان، از بین لیست تگ های زیر، مرتبط ترین موارد به درخواست کاربر را انتخاب کن.
    اگر کاربر در درخواست خود تعداد مکان هایی که میخواهد ببیند را ذکر کرد، آن را هم در متغیر total_cont بیاور.
    لیست تگ های مکان های تاریخی:[\n{}]\n
    لیست تگ های موزه ها:[\n{}]\n
    لیست تگ های رستوران ها:[\n{}]\n",
    neshan_history.tag_pool.join("\n"),
    neshan_museum.tag_pool.join("\n"),
    neshan_restaurant.tag_pool.join("\n"),
  );

  // -- Build the new client with this client_config
  let client = Client::builder()
    .with_config(ClientConfig::default().with_chat_options(
      ChatOptions::default().with_response_format(ChatResponseFormat::JsonSpec(JsonSpec::new(
        "items",
        json!(
          {
            "type": "object",
            "properties": {
              "place_infos": {
                "type": "array",
                "items": {
                  "type": "object",
                  "properties": {
                    "tags": {
                      "type": "array",
                      "items": {
                        "type": "string"
                      }
                    },
                    "place_type": {
                      "type": "string",
                      "enum": [
                        "موزه",
                        "مکان تاریخی",
                        "رستوران"
                      ]
                    }
                  }
                }
              },
              "total_count": {
                "type": "integer"
              }
            },
            "required": [
              "place_infos"
            ]
          }
        ),
      ))), // .with_temperature(0.0)
           // .with_top_p(0.99),
    ))
    .build();
  // -- Build the chat request
  let chat_req =
    ChatRequest::new(vec![ChatMessage::system(system_prompt), ChatMessage::user(name)]);
  println!("Question:\n{:#?}", chat_req);

  // -- Build the chat request options (used per execution chat)
  let chat_res = client
    .exec_chat(MODEL, chat_req.clone(), Some(&ChatOptions::default().with_max_tokens(1000)))
    .await
    .unwrap();
  println!("{:#?}", chat_res);
  ///////

  use vec_embed_store::{EmbeddingEngineOptions, EmbeddingsDb, SimilaritySearch, TextChunk};

  let prompt_analyse =
    serde_json::from_str::<PromptAnalyse>(chat_res.content_text_as_str().unwrap()).unwrap();
  println!("🟣🟣🟣\n{prompt_analyse:#?}\n🟣🟣🟣");
  // let neshan_history = serde_json::from_str::<Model>(include_str!(
  //   "taged_items/neshan_history_results_unique_with_tags.json"
  // ))
  // .unwrap();
  // let neshan_museum = serde_json::from_str::<Model>(include_str!(
  //   "taged_items/neshan_museum_results_unique_with_tags.json"
  // ))
  // .unwrap();
  // let neshan_restaurant = serde_json::from_str::<Model>(include_str!(
  //   "taged_items/neshan_restaurant_results_unique_with_tags.json"
  // ))
  // .unwrap();
  // let prompt_analyse = PromptAnalyse {
  //   place_infos: vec![
  //     PlaceInfo {
  //       place_type: PlaceType::Historical,
  //       tags: vec!["دوره صفویه".to_string(), "دوره تاریخی نامشخص".to_string()],
  //     },
  //     PlaceInfo {
  //       place_type: PlaceType::Restaurant, tags: vec!["دیزی سرای سید".to_string()]
  //     },
  //     PlaceInfo { place_type: PlaceType::Museum, tags: vec!["ساعت".to_string()] },
  //   ],
  //   total_count: Some(2),
  // };
  let mut res = prompt_analyse
    .place_infos
    .into_iter()
    .map(|info| {
      // println!("{info:#?}");
      match info.place_type {
        PlaceType::Museum => neshan_museum
          .items
          .iter()
          .map(|place| PlaceScoring {
            score: place.tags.iter().filter(|tag| info.tags.contains(tag)).count(),
            place: place.clone(),
          })
          .filter(|place_scoring| place_scoring.score > 0)
          .collect::<Vec<_>>(),
        PlaceType::Historical => neshan_history
          .items
          .iter()
          .map(|place| PlaceScoring {
            score: place.tags.iter().filter(|tag| info.tags.contains(tag)).count(),
            place: place.clone(),
          })
          .filter(|place_scoring| place_scoring.score > 0)
          .collect::<Vec<_>>(),
        PlaceType::Restaurant => neshan_restaurant
          .items
          .iter()
          .map(|place| PlaceScoring {
            score: place.tags.iter().filter(|tag| info.tags.contains(tag)).count(),
            place: place.clone(),
          })
          .filter(|place_scoring| place_scoring.score > 0)
          .collect::<Vec<_>>(),
      }
    })
    .filter(|palce_scoring| palce_scoring.len() > 0)
    .collect::<Vec<_>>();
  // println!("{res:#?}");
  res
    .into_iter()
    .inspect(|f| println!("🟡{f:#?}"))
    .multi_cartesian_product()
    .inspect(|f| println!("🙏{f:#?}"))
    .sorted_by(|a, b| {
      let max_distance = 20.0; // km

      let dist_a = (0..a.len())
        .map(|i| distance_haversine(&a[i].place.location, &a[(i + 1) % a.len()].place.location))
        .sum::<f64>();
      let dist_b = (0..b.len())
        .map(|i| distance_haversine(&b[i].place.location, &b[(i + 1) % b.len()].place.location))
        .sum::<f64>();

      // Normalize distances to 0-1 scale
      let norm_dist_a = dist_a / max_distance;
      let norm_dist_b = dist_b / max_distance;

      // Normalize scores to 0-1 scale (assuming higher score is better)
      let score_a =
        a.iter().map(|item| (item.score - 1) as f64 / 3.0).sum::<f64>() / a.len() as f64;
      let score_b =
        b.iter().map(|item| (item.score - 1) as f64 / 3.0).sum::<f64>() / b.len() as f64;

      // Combine normalized distance and score. Here we use subtraction because lower distance and higher score are better.
      // If you want to make distance and score equally important, you might adjust the weights.
      let combined_a = (0.7 * norm_dist_a) - (0.3 * score_a);
      let combined_b = (0.7 * norm_dist_b) - (0.3 * score_b);
      combined_a.partial_cmp(&combined_b).unwrap_or(Equal)
    })
    .next()
    .unwrap()
    .into_iter()
    .map(|place_scoring| place_scoring.place)
    .collect::<Vec<Place>>()
  // panic!("HEEEEY");
  // println!("✅✅✅✅\n{final_places:#?}\n✅✅✅✅");
  // final_places
}

#[derive(Debug, Serialize)]
struct Message {
  role: String,
  content: String,
}
#[derive(Debug, Serialize)]
struct CompletionRequest {
  model: String,
  messages: Vec<Message>,
  temperature: u32,
  response_format: HashMap<String, String>,
}

#[tauri::command]
fn answer(search_text: String) -> String {
  return "salam".to_string();
  // let api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY must be set");
  // let client = Client::new();
  // let messages = vec![Message {
  //   role: "user".to_string(),
  //   content: format!(r#"Translate this text to English:\n{search_text}"#),
  // }];
  // println!("\n\n🔴🔴🔴🔴🔴🔴🔴  Message:\n{messages:#?}\n🔴🔴🔴🔴🔴🔴\n\n");
  // let request = CompletionRequest {
  //   temperature: 0,
  //   response_format: HashMap::new(),
  //   model: "mixtral-8x7b-32768".to_string(),
  //   messages,
  // };
  // let json_response = client
  //   .post("https://api.groq.com/openai/v1/chat/completions")
  //   .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
  //   .json(&request)
  //   .send()
  //   .await
  //   .unwrap()
  //   .json::<Value>()
  //   .await
  //   .unwrap();
  // let translated_search_text = json_response["choices"][0]["message"]["content"].as_str().unwrap();
  // use vec_embed_store::{EmbeddingEngineOptions, EmbeddingsDb, SimilaritySearch, TextChunk};
  // let embedding_engine_options = EmbeddingEngineOptions {
  //   // model_name: BGESmallENV15, // see https://docs.rs/fastembed/latest/fastembed/enum.EmbeddingModel.html
  //   cache_dir: PathBuf::from("cache"),
  //   show_download_progress: true,
  //   ..Default::default()
  // };
  // // Create a new instance of EmbeddingsDb
  // let embed_db = EmbeddingsDb::new("fastembed-2", embedding_engine_options).await.unwrap();

  // // Define a text for similarity search
  // // let search_text = &input[1..].iter().cloned().collect::<String>();

  // // Perform a similarity search
  // let search_results = embed_db
  //   .get_similar_to(translated_search_text)
  //   .limit(2)
  //   .threshold(0.8)
  //   .execute()
  //   .await
  //   .unwrap();

  // println!("Similarity search results:");
  // for result in search_results.iter() {
  //   println!("ID: {}, Text: {}, Distance: {}", result.id, result.text, result.distance);
  // }

  // let messages = vec![Message {
  //   role: "user".to_string(),
  //   content: format!(
  //     "Response Language: Persian, Context: {}, Query: {search_text}",
  //     search_results.into_iter().map(|f| f.text).join("\n"),
  //   ),
  // }];
  // println!("\n\n🔴🔴🔴🔴🔴🔴🔴  Message:\n{messages:#?}\n🔴🔴🔴🔴🔴🔴\n\n");
  // let request = CompletionRequest {
  //   response_format: HashMap::new(),
  //   temperature: 1,
  //   model: "llama3-8b-8192".to_string(),
  //   messages,
  // };
  // let json_response = client
  //   .post("https://api.groq.com/openai/v1/chat/completions")
  //   .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
  //   .json(&request)
  //   .send()
  //   .await
  //   .unwrap()
  //   .json::<Value>()
  //   .await
  //   .unwrap();
  // let text_response = json_response["choices"][0]["message"]["content"].as_str().unwrap();
  // println!("✅✅✅✅\n{text_response}\n✅✅✅✅");
  // text_response.to_string()
}
