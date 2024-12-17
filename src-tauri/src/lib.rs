#![allow(unused)]
use rand::{rngs::StdRng, Rng as _, SeedableRng};
use std::{collections::HashMap, env, fmt::Debug, path::PathBuf};
use tauri_plugin_log::{Target, TargetKind};
use tokio::sync::Mutex;

use dotenv::dotenv;
use iter_tools::Itertools;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Location {
  x: f64,
  y: f64,
  z: String,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct ResponseModel {
  names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Place {
  title: String,
  category: String,
  r#type: String,
  region: String,
  #[serde(default)]
  neighbourhood: String,
  location: Location,
  #[serde(default)]
  tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct Model {
  items: Vec<Place>,
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

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn greet(name: String) -> Vec<Place> {
  let mut rng: StdRng = StdRng::from_entropy();

  let random_points = (0..20000)
    .map(|_| {
      let lat = rng.gen_range(35.60..=35.80);
      let lng = rng.gen_range(51.20..=51.50);
      [lat, lng]
    })
    .collect::<Vec<_>>();
  use rstar::RTree;
  let tree = RTree::bulk_load(random_points);
  let n = Instant::now();
  let random_points = (0..10)
    .map(|_| {
      let lat = rng.gen_range(35.60..=35.80);
      let lng = rng.gen_range(51.20..=51.50);
      [lat, lng]
    })
    .for_each(|f| {
      let nearest_neighbors = tree.nearest_neighbor_iter(&f).collect::<Vec<_>>();

      println!("{nearest_neighbors:#?}");
    });

  println!("Time Passed: {}", n.elapsed().as_millis());

  // Lat: 35.60 - 35.80
  // Long: 51.20 - 51.50

  use vec_embed_store::{EmbeddingEngineOptions, EmbeddingsDb, SimilaritySearch, TextChunk};

  let api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY must be set");
  let client = Client::new();

  // let messages = vec![Message {
  //   role: "user".to_string(),
  //   content: format!(r#"Translate this text to English:\n{name}"#),
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

  let messages = vec![Message {
    role: "user".to_string(),
    content: format!(
      r#"Only responde with valid Json without any wrapper nor introduction nor summary.
      Detect which place types the query request and how many of them.
      Then devide the query into slices based on the place_type and put the related slice in the json struct.
      Response format: 
      {{
        items: [
          {{
            place_type: "restaurant",
            count: 1,
            prompt_slice: ""
          }}
        ],
      }}
      Possible place_types: ["restaurant","museum","historical"],
      Query: {name}"#,
    ),
  }];
  println!("\n\n🔴🔴🔴🔴🔴🔴🔴  Message:\n{messages:#?}\n🔴🔴🔴🔴🔴🔴\n\n");
  let request = CompletionRequest {
    response_format: HashMap::from([("type".to_string(), "json_object".to_string())]),
    temperature: 0,
    model: "gemma2-9b-it".to_string(),
    messages,
  };
  let json_response = client
    .post("https://api.groq.com/openai/v1/chat/completions")
    .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
    .json(&request)
    .send()
    .await
    .unwrap()
    .json::<Value>()
    .await
    .unwrap();
  let prompt_classification_array = serde_json::from_str::<PromptClassificationArray>(&format!(
    "{}",
    json_response["choices"][0]["message"]["content"].as_str().unwrap()
  ))
  .unwrap();
  println!("🟣🟣🟣\n{prompt_classification_array:#?}\n🟣🟣🟣");
  let mut final_places = Vec::<Place>::new();
  for prompt_item in prompt_classification_array.items {
    match prompt_item.place_type.as_str() {
      "restaurant" => {
        println!("🟡 Finding restaurants");
        let embedding_engine_options = EmbeddingEngineOptions {
          // model_name: BGESmallENV15, // see https://docs.rs/fastembed/latest/fastembed/enum.EmbeddingModel.html
          cache_dir: PathBuf::from("cache-restaurant"),
          show_download_progress: true,
          ..Default::default()
        };
        // Create a new instance of EmbeddingsDb
        let embed_db =
          EmbeddingsDb::new("fastembed-restaurant", embedding_engine_options).await.unwrap();

        // Define a text for similarity search
        // let search_text = &input[1..].iter().cloned().collect::<String>();

        // Perform a similarity search
        let search_results = embed_db
          .get_similar_to(&prompt_item.prompt_slice)
          .limit(prompt_item.count as usize)
          .threshold(0.8)
          .execute()
          .await
          .unwrap();
        final_places.extend(
          search_results
            .iter()
            .inspect(|f| println!("{f:#?}"))
            .map(|f| serde_json::from_str::<Place>(&f.text).unwrap()),
        );

        // for result in search_results.iter() {
        //   // let model = serde_json::from_str::<Model>(&result.text);
        //   // println!("🟣🟣🟣place:🟣🟣🟣\n{:#?}", model);
        //   println!("ID: {},\n Text: {},\n Distance: {}\n", result.id, result.text, result.distance);
        // }
      }
      "museum" => {
        println!("🟡 Finding museum");
        let embedding_engine_options = EmbeddingEngineOptions {
          // model_name: BGESmallENV15, // see https://docs.rs/fastembed/latest/fastembed/enum.EmbeddingModel.html
          cache_dir: PathBuf::from("cache-museum"),
          show_download_progress: true,
          ..Default::default()
        };
        // Create a new instance of EmbeddingsDb
        let embed_db =
          EmbeddingsDb::new("fastembed-museum", embedding_engine_options).await.unwrap();

        // Define a text for similarity search
        // let search_text = &input[1..].iter().cloned().collect::<String>();

        // Perform a similarity search
        let search_results = embed_db
          .get_similar_to(&prompt_item.prompt_slice)
          .limit(prompt_item.count as usize)
          .threshold(0.8)
          .execute()
          .await
          .unwrap();
        final_places.extend(
          search_results
            .iter()
            .inspect(|f| println!("{:#?}", f.text))
            .map(|f| serde_json::from_str::<Place>(&f.text).unwrap()),
        );
        // for result in search_results.iter() {
        //   // let model = serde_json::from_str::<Model>(&result.text);
        //   // println!("🟣🟣🟣place:🟣🟣🟣\n{:#?}", model);
        //   println!("ID: {},\n Text: {},\n Distance: {}\n", result.id, result.text, result.distance);
        // }
      }
      "historical" => {
        println!("🟡 Finding historical");
        let embedding_engine_options = EmbeddingEngineOptions {
          // model_name: BGESmallENV15, // see https://docs.rs/fastembed/latest/fastembed/enum.EmbeddingModel.html
          cache_dir: PathBuf::from("cache-history"),
          show_download_progress: true,
          ..Default::default()
        };
        // Create a new instance of EmbeddingsDb
        let embed_db =
          EmbeddingsDb::new("fastembed-history", embedding_engine_options).await.unwrap();

        // Define a text for similarity search
        // let search_text = &input[1..].iter().cloned().collect::<String>();

        // Perform a similarity search
        let search_results = embed_db
          .get_similar_to(&prompt_item.prompt_slice)
          .limit(prompt_item.count as usize)
          .threshold(0.8)
          .execute()
          .await
          .unwrap();
        final_places.extend(
          search_results
            .iter()
            .inspect(|f| println!("{f:#?}"))
            .map(|f| serde_json::from_str::<Place>(&f.text).unwrap()),
        );
        // for result in search_results.iter() {
        //   // let model = serde_json::from_str::<Model>(&result.text);
        //   // println!("🟣🟣🟣place:🟣🟣🟣\n{:#?}", model);
        //   println!("ID: {},\n Text: {},\n Distance: {}\n", result.id, result.text, result.distance);
        // }
      }
      _ => unreachable!(),
    }
  }

  // let embedding_engine_options = EmbeddingEngineOptions {
  //   // model_name: BGESmallENV15, // see https://docs.rs/fastembed/latest/fastembed/enum.EmbeddingModel.html
  //   cache_dir: PathBuf::from("cache-3"),
  //   show_download_progress: true,
  //   ..Default::default()
  // };
  // // Create a new instance of EmbeddingsDb
  // let embed_db = EmbeddingsDb::new("fastembed-3", embedding_engine_options).await.unwrap();

  // // Define a text for similarity search
  // // let search_text = &input[1..].iter().cloned().collect::<String>();

  // // Perform a similarity search
  // let search_results =
  //   embed_db.get_similar_to(&name).limit(5).threshold(0.8).execute().await.unwrap();

  // println!("Similarity search results:");
  // for result in search_results.iter() {
  //   // let model = serde_json::from_str::<Model>(&result.text);
  //   // println!("🟣🟣🟣place:🟣🟣🟣\n{:#?}", model);
  //   println!("ID: {}, Text: {}, Distance: {}", result.id, result.text, result.distance);
  // }

  // let messages = vec![Message {
  //   role: "user".to_string(),
  //   content: format!(
  //     r#"Only responde with valid Json without any wrapper nor introduction nor summary.
  //     if there is nothing related, just return null.
  //     Response format: {{"names":<string>[]}},
  //     Context: {}.
  //     Query: {name}"#,
  //     search_results.into_iter().map(|f| f.text).join("\n"),
  //   ),
  // }];
  // println!("\n\n🔴🔴🔴🔴🔴🔴🔴  Message:\n{messages:#?}\n🔴🔴🔴🔴🔴🔴\n\n");
  // let request = CompletionRequest {
  //   response_format: HashMap::from([("type".to_string(), "json_object".to_string())]),
  //   temperature: 0,
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
  // println!("{}", &json_response);

  // // let text_response = json_response["choices"][0]["message"]["content"].as_str().unwrap();
  // let response_model = serde_json::from_str::<ResponseModel>(&format!(
  //   "{}",
  //   json_response["choices"][0]["message"]["content"].as_str().unwrap()
  // ))
  // .unwrap();
  // println!("✅✅✅✅\n{response_model:#?}\n✅✅✅✅");

  println!("✅✅✅✅\n{final_places:#?}\n✅✅✅✅");
  final_places
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
