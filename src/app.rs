#![allow(unused)]

use dotenvy_macro::dotenv;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ChatResponseFormat, JsonSpec};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ClientConfig, ModelIden, ServiceTarget};

use iter_tools::Itertools;
use leptos::leptos_dom::logging::console_log;
use leptos::tachys::html::property::IntoProperty;
use reactive_stores::{OptionStoreExt as _, Store, StoreFieldIterator};
use serde_json::{json, to_value};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::iter::repeat;
use std::ops::Index;
use std::rc::Rc;
use std::str::FromStr;
use std::{
  cmp::Ordering::Equal,
  fs::read_to_string,
  hash::{Hash, Hasher},
  ops::Not,
};
use strum::{Display, EnumString, VariantArray, VariantNames};
use wasm_bindgen::prelude::*;
use web_sys::console::time;
use web_sys::js_sys::Math::random;
use web_sys::js_sys::{Array, JsString, Object, Reflect};

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use leptos::{either::Either, prelude::*, task::spawn_local};
use serde::{Deserialize, Serialize};
// use std::fmt::Display;
// use strum::{Display, EnumString};
// use strum::{VariantArray, VariantNames};
use wasm_bindgen::prelude::*;
use web_sys::{window, Window};
use web_sys::{InputEvent, MouseEvent, SubmitEvent};

// #[wasm_bindgen]
// extern "C" {
//   #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
//   async fn invoke(cmd: &str, args: JsValue) -> JsValue;
// }

// #[derive(Serialize, Deserialize)]
// struct AnswerArgs {
//   search_text: String,
// }
// const EPSILON: f64 = 1e-10;
// #[derive(Serialize, Deserialize)]
// struct GreetArgs {
//   name: String,
// }
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

#[derive(Hash, Eq, Debug, Serialize, Deserialize, Clone)]
struct PlaceScoring {
  place: Place,
  score: usize,
}

impl PartialEq for PlaceScoring {
  fn eq(&self, other: &Self) -> bool {
    let res = self.place == other.place && self.score == other.score;
    // console_log(&format!("{res}"));
    res
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
// impl PartialEq for Location {
//   fn eq(&self, other: &Self) -> bool {
//     // Define a small epsilon for floating-point comparison
//     const EPSILON: f64 = 1e-9; // Adjust this value based on your precision needs

//     // Check if the difference between coordinates is less than EPSILON
//     (self.x - other.x).abs() < EPSILON && (self.y - other.y).abs() < EPSILON
//   }
// }
// impl Eq for Location {}
// impl Hash for Location {
//   fn hash<H: Hasher>(&self, state: &mut H) {
//     // Scaling factor for converting float to integer for hashing
//     const SCALE_FACTOR: f64 = 1e9; // This should match or be related to EPSILON

//     // Convert to scaled integers to ensure hash stability for "equal" floats
//     let x_int = (self.x * SCALE_FACTOR).round() as i64;
//     let y_int = (self.y * SCALE_FACTOR).round() as i64;

//     // Hash the integers
//     x_int.hash(state);
//     y_int.hash(state);
//   }
// }

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct ResponseModel {
  names: Vec<String>,
}
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

impl Eq for Place {}

impl Hash for Place {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.title.hash(state);
    self.r#type.hash(state);
  }
}

impl PartialEq for Place {
  fn eq(&self, other: &Self) -> bool {
    let res = self.title == other.title && self.r#type == other.r#type;
    // console_log(&format!("{} vs {}", self.title, other.title));
    res
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct Model {
  tag_pool: Vec<String>,
  items: Vec<Place>,
}

#[derive(Debug, Store, PartialEq, Eq, Hash)]
pub struct SelectPlace {
  checked: bool,
  place_type: PlaceType,
}
#[derive(Debug, Store, PartialEq, Eq, Hash)]
pub struct State {
  pub num_places: String,
  pub budget: String,
  pub time: String,
  #[store(key:PlaceType=|n|n.place_type)]
  pub select_places: Vec<SelectPlace>,
  pub result: String,
}
impl Default for State {
  fn default() -> Self {
    Self {
      num_places: "5".to_string(),
      budget: "1000000".to_string(),
      time: "4".to_string(),
      select_places: PlaceType::VARIANTS
        .iter()
        .map(|f| SelectPlace { checked: true, place_type: f.to_owned() })
        .collect_vec(),
      result: "".to_string(),
    }
  }
}

fn shuffle_vec<T: Debug>(vec: &mut Vec<T>) {
  let len = vec.len();
  for i in 0..len {
    let j = (web_sys::js_sys::Math::random() * (len as f64)) as usize;
    vec.swap(i, j);
  }
  // console_log(&format!("{vec:#?}"));
}

fn select_places(state: &State, places: Vec<Place>) -> Vec<String> {
  // let mut rng = rand::rngs::ThreadRng::default();
  // web_sys::js_sys::Math::random()
  let selected_types: HashSet<_> = state.select_places.iter().map(|sp| &sp.place_type).collect();
  let mut filtered_places: Vec<_> =
    places.into_iter().filter(|place| selected_types.contains(&place.r#type)).collect();

  shuffle_vec(&mut filtered_places);

  let mut selected_places = Vec::new();
  let mut place_types = HashSet::new();

  for place in filtered_places {
    if selected_places.len().to_string() == state.num_places {
      break;
    }
    if !place_types.contains(&place.r#type) {
      place_types.insert(place.r#type.clone());
      selected_places.push(place);
    }
  }
  selected_places.iter().map(|f| f.title.clone()).collect_vec()
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
#[component]
pub fn App() -> impl IntoView {
  console_log(&"App is Called");
  let options = Object::new();
  Reflect::set(&options, &JsValue::from_str("mapType"), &JsValue::from_str("neshanVector"))
    .unwrap();
  Reflect::set(&options, &JsValue::from_str("container"), &JsValue::from_str("map")).unwrap();
  Reflect::set(&options, &JsValue::from_str("zoom"), &JsValue::from_f64(10.0)).unwrap();
  Reflect::set(&options, &JsValue::from_str("pitch"), &JsValue::from_f64(0.0)).unwrap();
  Reflect::set(
    &options,
    &JsValue::from_str("center"),
    &JsValue::from(Array::of2(&JsValue::from_f64(51.391173), &JsValue::from_f64(35.700954))),
  )
  .unwrap();
  Reflect::set(&options, &JsValue::from_str("minZoom"), &JsValue::from_f64(2.0)).unwrap();
  Reflect::set(&options, &JsValue::from_str("maxZoom"), &JsValue::from_f64(21.0)).unwrap();
  Reflect::set(&options, &JsValue::from_str("trackResize"), &JsValue::from_bool(true)).unwrap();
  Reflect::set(
    &options,
    &JsValue::from_str("mapKey"),
    &JsValue::from_str(dotenv!("NESHAN_API_KEY")),
  )
  .unwrap();
  Reflect::set(&options, &JsValue::from_str("poi"), &JsValue::from_bool(false)).unwrap();
  Reflect::set(&options, &JsValue::from_str("traffic"), &JsValue::from_bool(false)).unwrap();
  let map_controller_options = Object::new();
  Reflect::set(&map_controller_options, &JsValue::from_str("show"), &JsValue::from_bool(true))
    .unwrap();
  Reflect::set(
    &map_controller_options,
    &JsValue::from_str("position"),
    &JsValue::from_str("bottom-left"),
  )
  .unwrap();
  Reflect::set(
    &options,
    &JsValue::from_str("mapTypeControllerOptions"),
    &JsValue::from(map_controller_options),
  )
  .unwrap();
  console_log(&"7");
  console_log(&"Map was created");
  let places = RwSignal::new(Vec::<Place>::new());
  let markers = StoredValue::new_local(Vec::<Marker>::new());
  let map_ref: RwSignal<Option<Map>, LocalStorage> = RwSignal::new_local(None);
  Effect::new(move |old_places: Option<Vec<Place>>| {
    if let Some(old_places) = old_places {
      markers.get_value().iter().for_each(Marker::remove);
      markers.set_value(
        places
          .get()
          .iter()
          .map(|place| {
            Marker::newMarker().setLngLat(&JsValue::from(Array::of2(
              &JsValue::from_f64(place.location.x),
              &JsValue::from_f64(place.location.y),
            )))
          })
          .inspect(|marker| {
            marker.addTo(map_ref.get().as_ref().unwrap());
          })
          .collect_vec(),
      );
    }
    places.get()
  });
  request_animation_frame(move || {
    map_ref.set(Some(Map::newMap(&JsValue::from(options))));
  });

  let state = Store::new(State::default());
  let prompt_text = RwSignal::new("".to_string());
  let answer_text = RwSignal::new("".to_string());
  let answer = move |ev: MouseEvent| {
    ev.prevent_default();

    // console_log("Hi From outside of the spaawn");
    spawn_local(async move {
      // console_log("Hi");
      // let args = serde_wasm_bindgen::to_value(&GreetArgs { name: prompt_text.get() }).unwrap();
      // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
      let new_msg = do_the_job(prompt_text.get()).await;
      console_log(&format!("{new_msg:#?}"));
      places.set(new_msg);
    });
  };
  console_log(&"About to rendering view");
  view! {
    <div>
      <div id="map" style="height:500px;width:100%;"></div>

      <textarea name="prompt" id="prompt" bind:value=prompt_text />
      <button on:click=answer>"Generate Suggestion"</button>
      {answer_text}
      <PlacesList places />
    </div>
  }
}

#[component]
fn PlacesList(#[prop(into)] places: Signal<Vec<Place>>) -> impl IntoView {
  view! {
    <div class="wrapper">
      <ol class="c-st1per">
        {move || {
          places
            .get()
            .into_iter()
            .map(|place| {
              view! {
                <li class="c-stepper__item">
                  <div class="c-stepper__content">
                    <PlaceCard place />
                  </div>
                </li>
              }
            })
            .collect_view()
        }}
      </ol>
    </div>
  }
}

#[component]
fn PlaceCard(place: Place) -> impl IntoView {
  view! {
    <div class="card">
      <h2>{place.title}</h2>
      <p>
        <strong>"Category:"</strong>
        {place.category}
      </p>
      <p>
        <strong>"Type:"</strong>
        {place.r#type.to_string()}
      </p>
      <p>
        <strong>"Region:"</strong>
        {place.region}
      </p>
      <p>
        <strong>"Neighbourhood:"</strong>
        {place.neighbourhood}
      </p>
      <p>
        <strong>"Location:"</strong>
        {format!("{}, {}", place.location.x, place.location.y)}
      </p>
      <p>
        <strong>Tags:</strong>
        {place.tags.join(", ")}
      </p>
    </div>
  }
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
  //   .collect_vec();
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
  //     let nearest_neighbors = tree.nearest_neighbor_iter(&f).collect_vec();

  //     println!("{nearest_neighbors:#?}");
  //   });

  // println!("Time Passed: {}", n.elapsed().as_millis());

  // Lat: 35.60 - 35.80
  // Long: 51.20 - 51.50
  let mut neshan_history = serde_json::from_str::<Model>(include_str!(
    "taged_items/neshan_history_results_unique_with_tags.json"
  ))
  .unwrap();
  let mut neshan_museum = serde_json::from_str::<Model>(include_str!(
    "taged_items/neshan_museum_results_unique_with_tags.json"
  ))
  .unwrap();
  let mut neshan_restaurant = serde_json::from_str::<Model>(include_str!(
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
    ChatRequest::new(vec![ChatMessage::system(system_prompt), ChatMessage::user(name)]);
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
        .iter()
        .filter(|place| {
          place.r#type == info.place_type && place.tags.iter().any(|tag| info.tags.contains(tag))
        })
        .map(|place| PlaceScoring {
          score: place.tags.iter().filter(|tag| info.tags.contains(tag)).count(),
          place: place.clone(),
        })
        .collect_vec()
    })
    // .collect::<Vec<Vec<PlaceScoring>>>()
    // .into_iter()
    // .unique()
    // .inspect(|f| console_log(&format!("{f:#?}")))
    // .collect::<Vec<Vec<PlaceScoring>>>()
    // .into_iter()
    .multi_cartesian_product()
    .map(|place_scoring_list| place_scoring_list.into_iter().unique().collect_vec())
    .unique()
    .sorted_by(|a, b| {
      if a.len() != b.len() {
        return b.len().cmp(&a.len());
      }
      let max_distance = 20.0; // km
      let dist_a = a
        .iter()
        .circular_tuple_windows()
        .map(|(item1, item2)| distance_haversine(&item1.place.location, &item2.place.location))
        .sum::<f64>();
      let dist_b = b
        .iter()
        .circular_tuple_windows()
        .map(|(item1, item2)| distance_haversine(&item1.place.location, &item2.place.location))
        .sum::<f64>();

      // Normalize distances to 0-1 scale
      let norm_dist_a = dist_a / max_distance;
      let norm_dist_b = dist_b / max_distance;

      // Normalize scores to 0-1 scale (assuming higher score is better)
      let score_a = a.iter().map(|item| (item.score) as f64 / 3.0).sum::<f64>() / a.len() as f64;
      let score_b = b.iter().map(|item| (item.score) as f64 / 3.0).sum::<f64>() / b.len() as f64;

      // Combine normalized distance and score. Here we use subtraction because lower distance and higher score are better.
      // If you want to make distance and score equally important, you might adjust the weights.
      let combined_a = (0.1 * norm_dist_a) - (0.9 * score_a);
      let combined_b = (0.1 * norm_dist_b) - (0.9 * score_b);
      combined_a.partial_cmp(&combined_b).unwrap_or(Equal)
    })
    .inspect(|f| console_log(&format!("{f:#?}")))
    .collect_vec()
    .first()
    // .next()
    .unwrap()
    .into_iter()
    .map(|place_scoring| place_scoring.place.clone())
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
