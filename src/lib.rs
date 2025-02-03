pub mod components;

use better_default::Default;
use dotenvy_macro::dotenv;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ChatResponseFormat, JsonSpec};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ClientConfig, ModelIden, ServiceTarget};
use iter_tools::Itertools;
use leptos::leptos_dom::logging::console_log;
use reactive_stores::{Field, Store};
use serde_json::json;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::iter::repeat;
use strum::{Display, EnumString, VariantArray};
use wasm_bindgen::prelude::*;
use web_sys::js_sys::JsString;

use chrono::{DateTime, Local};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceInfo {
  pub place_type: PlaceType,
  pub tags: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PromptAnalyses {
  // entry_point: Option<Location>,
  pub place_infos: Vec<PlaceInfo>,
  pub total_count: Option<u32>,
}

#[derive(Hash, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct PlaceScoring {
  place: Place,
  score: usize,
}

impl PartialEq for PlaceScoring {
  fn eq(&self, other: &Self) -> bool {
    self.place == other.place && self.score == other.score
  }
}
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
  // VariantNames,
)]
pub enum PlaceType {
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
pub struct Location {
  pub x: f64,
  pub y: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Place {
  pub title: String,
  pub category: String,
  pub r#type: PlaceType,
  pub region: String,
  #[serde(default)]
  pub neighbourhood: String,
  pub location: Location,
  #[serde(default)]
  pub tags: Vec<String>,
}

impl Eq for Place {}

impl Hash for Place {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.title.hash(state);
    self.r#type.hash(state);
  }
}

impl PartialEq for Place {
  fn eq(&self, other: &Self) -> bool {
    self.title == other.title && self.r#type == other.r#type
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NeshanDataModel {
  pub tag_pool: Vec<String>,
  pub items: Vec<Place>,
}

#[derive(Default, Store)]
pub struct State {
  pub prompt_text: String, // should be in session
  #[store(key: DateTime<Local> = |session| session.date_created)]
  pub sessions: Vec<Session>,
  #[default(true)]
  pub is_sidebar_visible: bool,
  pub answering: bool,
}
// pub trait StateExt {
//   fn selected_session(&self) -> Option<Field<Session>>;
// }
// impl StateExt for Store<State> {
//   fn selected_session(&self) -> Option<Field<Session>> {
//     self.with(|f| f.selected_session)
//   }
// }
// impl Debug for State {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     f.debug_struct("State")
//       .field("prompt_text", &self.prompt_text)
//       .field("sessions", &self.sessions)
//       .field("selected_session", &"Not Implemented")
//       .finish()
//   }
// }

// impl Hash for State {
//   fn hash<H: Hasher>(&self, state: &mut H) {
//     self.prompt_text.hash(state);
//     self.sessions.hash(state);
//   }
// }

// impl Eq for State {}

// impl PartialEq for State {
//   fn eq(&self, other: &Self) -> bool {
//     self.prompt_text == other.prompt_text && self.sessions == other.sessions
//   }
// }

#[derive(Store, Clone)]
pub struct Session {
  pub date_created: DateTime<Local>,
  pub title: String,
  pub suggestions: Vec<Suggestion>,
  // #[store(skip)]
  // selected_suggestion: Option<Field<Suggestion>>,
}

impl Debug for Session {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Session")
      .field("date_created", &self.date_created)
      .field("title", &self.title)
      .field("suggestions", &self.suggestions)
      // .field("selected_session", &"Not Implemented")
      .finish()
  }
}
// pub trait SessionExt {
//   fn selected_suggestion(&self) -> Option<Field<Suggestion>>;
// }
// impl SessionExt for Field<Session> {
//   fn selected_suggestion(&self) -> Option<Field<Suggestion>> {
//     self.with(|f| f.selected_suggestion)
//   }
// }

impl Hash for Session {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.date_created.hash(state);
  }
}

impl PartialEq for Session {
  fn eq(&self, other: &Self) -> bool {
    self.date_created == other.date_created
  }
}

#[derive(Debug, Store, PartialEq, Eq, Hash, Clone)]
pub struct Suggestion {
  places: Vec<Place>,
  selected_place: Place,
}
pub trait SuggestionExt {
  fn next(&self);
  fn prev(&self);
}
impl SuggestionExt for Field<Suggestion> {
  fn next(&self) {
    let index = self
      .places()
      .with(|p| p.iter().position(|f| self.selected_place().with(|s| *s == *f)).unwrap())
      as isize;
    let len = self.places().with(Vec::len);
    let new_index = ((index + 1) % len as isize) as usize % len;
    self.selected_place().set(self.places().with(|f| f[new_index].clone()));
  }
  fn prev(&self) {
    let index = self
      .places()
      .with(|p| p.iter().position(|f| self.selected_place().with(|s| *s == *f)).unwrap())
      as isize;
    let len = self.places().with(Vec::len);
    let new_index = ((index - 1) % len as isize) as usize % len;
    self.selected_place().set(self.places().with(|f| f[new_index].clone()));
  }
}

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = nmp_mapboxgl)]
  #[derive(Clone)]
  type Map;
  #[wasm_bindgen(constructor, js_namespace = nmp_mapboxgl)]
  fn newMap(options: &JsValue) -> Map;
  #[wasm_bindgen(method, js_namespace = nmp_mapboxgl)]
  fn addTo(this: &Map, container: &JsValue);
  #[wasm_bindgen(js_namespace = nmp_mapboxgl)]
  #[derive(Clone)]
  type Marker;
  #[wasm_bindgen(constructor, js_namespace = nmp_mapboxgl)]
  fn newMarker() -> Marker;
  #[wasm_bindgen(method, js_namespace = nmp_mapboxgl)]
  fn setLngLat(this: &Marker, lng_lat: &JsValue) -> Marker;
  #[wasm_bindgen(method, js_namespace = nmp_mapboxgl)]
  fn addTo(this: &Marker, map: &Map) -> Marker;
  #[wasm_bindgen(method, js_namespace = nmp_mapboxgl)]
  fn remove(this: &Marker);
  #[wasm_bindgen(js_namespace = nmp_mapboxgl)]
  type LngLat;
  #[wasm_bindgen(method, js_namespace = nmp_mapboxgl)]
  fn toString(this: &LngLat) -> JsString;
  #[wasm_bindgen(method, js_namespace = nmp_mapboxgl)]
  fn getLngLat(this: &Marker) -> LngLat;

}

async fn ask_ai(prompt: String) -> Vec<Suggestion> {
  let neshan_history = serde_json::from_str::<NeshanDataModel>(include_str!(
    "taged_items/neshan_history_results_unique_with_tags.json"
  ))
  .unwrap();
  let neshan_museum = serde_json::from_str::<NeshanDataModel>(include_str!(
    "taged_items/neshan_museum_results_unique_with_tags.json"
  ))
  .unwrap();
  let neshan_restaurant = serde_json::from_str::<NeshanDataModel>(include_str!(
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
  // -- Build an auth_resolver and the AdapterConfig
  let target_resolver = ServiceTargetResolver::from_resolver_fn(
    |service_target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
      let ServiceTarget { model, .. } = service_target;
      let endpoint = Endpoint::from_static("https://generativelanguage.googleapis.com/v1beta/");
      let auth = AuthData::from_single(dotenv!("GEMINI_API_KEY"));
      let model = ModelIden::new(AdapterKind::Gemini, model.model_name);
      Ok(ServiceTarget { endpoint, auth, model })
    },
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
    .with_service_target_resolver(target_resolver)
    .build();
  // -- Build the chat request
  let chat_req =
    ChatRequest::new(vec![ChatMessage::system(system_prompt), ChatMessage::user(prompt)]);
  println!("Question:\n{:#?}", chat_req);

  // -- Build the chat request options (used per execution chat)
  let chat_res = client
    .exec_chat(
      "gemini-1.5-pro",
      chat_req.clone(),
      Some(&ChatOptions::default().with_max_tokens(1000)),
    )
    .await
    .unwrap();
  console_log(&format!("{:#?}", chat_res));
  ///////

  // use vec_embed_store::{EmbeddingEngineOptions, EmbeddingsDb, SimilaritySearch, TextChunk};

  let prompt_analyse =
    serde_json::from_str::<PromptAnalyses>(chat_res.content_text_as_str().unwrap()).unwrap();
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
  // neshan_history.items.swap_remove(neshan_history.items.iter().)
  let all_places = {
    let mut tmp = Vec::with_capacity(
      neshan_history.items.len() + neshan_museum.items.len() + neshan_restaurant.items.len(),
    );
    tmp.extend(neshan_history.items);
    tmp.extend(neshan_museum.items);
    tmp.extend(neshan_restaurant.items);
    tmp
  };

  prompt_analyse
    .place_infos
    .iter()
    .zip(repeat(&all_places))
    .map(|(info, all_places)| {
      all_places
        .into_iter()
        .filter(|place| {
          place.r#type == info.place_type && place.tags.iter().any(|tag| info.tags.contains(tag))
        })
        .cloned()
        // .map(|place| PlaceScoring {
        //   score: place.tags.iter().filter(|tag| info.tags.contains(tag)).count(),
        //   place: place.clone(),
        // })
        .collect_vec()
    })
    .map(|f| Suggestion { selected_place: f[0].clone(), places: f })
    .collect_vec()

  // let mut res = prompt_analyse
  //   .place_infos
  //   .into_iter()
  //   .map(|info| {
  //     // println!("{info:#?}");
  //     .filter(|place_scoring| place_scoring.score > 0)
  //     .map(|place| PlaceScoring {
  //           score: place.tags.iter().filter(|tag| info.tags.contains(tag)).count(),
  //           place: place.clone(),
  //         })
  //     match info.place_type {
  //       PlaceType::Museum => neshan_museum
  //         .items
  //         .iter()
  //         .map(|place| PlaceScoring {
  //           score: place.tags.iter().filter(|tag| info.tags.contains(tag)).count(),
  //           place: place.clone(),
  //         })
  //         .filter(|place_scoring| place_scoring.score > 0)
  //         .collect_vec(),
  //       PlaceType::Historical => neshan_history
  //         .items
  //         .iter()
  //         .map(|place| PlaceScoring {
  //           score: place.tags.iter().filter(|tag| info.tags.contains(tag)).count(),
  //           place: place.clone(),
  //         })
  //         .filter(|place_scoring| place_scoring.score > 0)
  //         .collect_vec(),
  //       PlaceType::Restaurant => neshan_restaurant
  //         .items
  //         .iter()
  //         .map(|place| PlaceScoring {
  //           score: place.tags.iter().filter(|tag| info.tags.contains(tag)).count(),
  //           place: place.clone(),
  //         })
  //         .filter(|place_scoring| place_scoring.score > 0)
  //         .collect_vec(),
  //     }
  //   })
  //   .filter(|palce_scoring| palce_scoring.len() > 0)
  //   .collect_vec();
  // // println!("{res:#?}");
  // res
  //   .into_iter()
  //   .inspect(|f| println!("🟡{f:#?}"))
  //   .multi_cartesian_product()
  //   .inspect(|f| println!("🙏{f:#?}"))
  //   .sorted_by(|a, b| {
  //     let max_distance = 20.0; // km

  //     let dist_a = (0..a.len())
  //       .map(|i| distance_haversine(&a[i].place.location, &a[(i + 1) % a.len()].place.location))
  //       .sum::<f64>();
  //     let dist_b = (0..b.len())
  //       .map(|i| distance_haversine(&b[i].place.location, &b[(i + 1) % b.len()].place.location))
  //       .sum::<f64>();

  //     // Normalize distances to 0-1 scale
  //     let norm_dist_a = dist_a / max_distance;
  //     let norm_dist_b = dist_b / max_distance;

  //     // Normalize scores to 0-1 scale (assuming higher score is better)
  //     let score_a =
  //       a.iter().map(|item| (item.score - 1) as f64 / 3.0).sum::<f64>() / a.len() as f64;
  //     let score_b =
  //       b.iter().map(|item| (item.score - 1) as f64 / 3.0).sum::<f64>() / b.len() as f64;

  //     // Combine normalized distance and score. Here we use subtraction because lower distance and higher score are better.
  //     // If you want to make distance and score equally important, you might adjust the weights.
  //     let combined_a = (0.7 * norm_dist_a) - (0.3 * score_a);
  //     let combined_b = (0.7 * norm_dist_b) - (0.3 * score_b);
  //     combined_a.partial_cmp(&combined_b).unwrap_or(Equal)
  //   })
  //   .next()
  //   .unwrap()
  //   .into_iter()
  //   .map(|place_scoring| place_scoring.place)
  //   .collect::<Vec<Place>>()
  // panic!("HEEEEY");
  // println!("✅✅✅✅\n{final_places:#?}\n✅✅✅✅");
  // final_places
}
fn _distance_haversine(loc1: &Location, loc2: &Location) -> f64 {
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
