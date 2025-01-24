#![allow(unused)]

use dotenvy_macro::dotenv;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ChatResponseFormat, JsonSpec};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ClientConfig, ModelIden, ServiceTarget};

use guards::{Mapped, MappedMutArc, Plain};
use iter_tools::Itertools;
use leptos::html::div;
use leptos::leptos_dom::logging::console_log;
use leptos::tachys::html::property::IntoProperty;
use reactive_stores::{
  AtKeyed, Field, KeyedSubfield, OptionStoreExt as _, Store, StoreField, StoreFieldIterator,
  Subfield,
};
use serde_json::{json, to_value};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::iter::repeat;
use std::ops::{Deref, Index};
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
use wasm_bindgen::prelude::*;
use web_sys::{window, Window};
use web_sys::{InputEvent, MouseEvent, SubmitEvent};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PlaceInfo {
  place_type: PlaceType,
  tags: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct PromptAnalyses {
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
    self.title == other.title && self.r#type == other.r#type
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct NeshanDataModel {
  tag_pool: Vec<String>,
  items: Vec<Place>,
}

#[derive(Default, Store)]
pub struct State {
  prompt_text: String, // should be in session
  #[store(key: DateTime<Local> = |session| session.date_created)]
  sessions: Vec<Session>,
  #[store(skip)]
  selected_session: Option<Field<Session>>,
}
pub trait StateExt {
  fn selected_session(&self) -> Option<Field<Session>>;
}
impl StateExt for Store<State> {
  fn selected_session(&self) -> Option<Field<Session>> {
    self.with(|f| f.selected_session)
  }
}
impl Debug for State {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("State")
      .field("prompt_text", &self.prompt_text)
      .field("sessions", &self.sessions)
      .field("selected_session", &"Not Implemented")
      .finish()
  }
}

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
  date_created: DateTime<Local>,
  title: String,
  suggestions: Vec<Suggestion>,
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
  #[store(key:Place = |place|place.clone())]
  places: Vec<Place>,
  selected_place: Place,
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

fn config_map(options: &Object) {
  Reflect::set(options, &JsValue::from_str("mapType"), &JsValue::from_str("neshanVector")).unwrap();
  Reflect::set(options, &JsValue::from_str("container"), &JsValue::from_str("map")).unwrap();
  Reflect::set(options, &JsValue::from_str("zoom"), &JsValue::from_f64(10.0)).unwrap();
  Reflect::set(options, &JsValue::from_str("pitch"), &JsValue::from_f64(0.0)).unwrap();
  Reflect::set(
    options,
    &JsValue::from_str("center"),
    &JsValue::from(Array::of2(&JsValue::from_f64(51.391173), &JsValue::from_f64(35.700954))),
  )
  .unwrap();
  Reflect::set(options, &JsValue::from_str("minZoom"), &JsValue::from_f64(2.0)).unwrap();
  Reflect::set(options, &JsValue::from_str("maxZoom"), &JsValue::from_f64(21.0)).unwrap();
  Reflect::set(options, &JsValue::from_str("trackResize"), &JsValue::from_bool(true)).unwrap();
  Reflect::set(
    options,
    &JsValue::from_str("mapKey"),
    &JsValue::from_str(dotenv!("NESHAN_API_KEY")),
  )
  .unwrap();
  Reflect::set(options, &JsValue::from_str("poi"), &JsValue::from_bool(false)).unwrap();
  Reflect::set(options, &JsValue::from_str("traffic"), &JsValue::from_bool(false)).unwrap();
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
    options,
    &JsValue::from_str("mapTypeControllerOptions"),
    &JsValue::from(map_controller_options),
  )
  .unwrap();
}

#[component]
pub fn App() -> impl IntoView {
  let map_options = Object::new();
  // config_map(&map_options);
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
    map_ref.set(Some(Map::newMap(&JsValue::from(map_options))));
  });

  let state = Store::new(State::default());
  // state.selected_session().;
  let answer = move |ev: MouseEvent| {
    spawn_local(async move {
      let answer = ask_ai(state.prompt_text().get()).await;
      // console_log(&format!("{:#?}", answer.clone()));
      state.selected_session().map(|ss| ss.write().suggestions = answer);
    });
  };

  let r = format!("{:#?}", state.path().into_iter().collect_vec());
  // state.read().selected_session.unwrap().date_created()
  // state.sessions().at_unkeyed(0).reader()
  // console_log(&"About to rendering view");
  // let r = AtKeyed::new(state.sessions(), state.sessions()..get()[0].date_created);

  // state.sessions().at_unkeyed(0).date_created().get()
  Effect::new(
    move |old_sessions: Option<
      KeyedSubfield<Store<State>, State, DateTime<Local>, Vec<Session>>,
    >| {
      if let Some(old_sessions) = old_sessions {
        // console_log(&format!(
        //   "{:#?}",
        //   state.sessions().at_unkeyed(0).path().into_iter().collect_vec()
        // ));
        if state.with_untracked(|f| {
          f.selected_session
            .get_untracked()
            .map(|ss| f.sessions.contains(&ss).not())
            .unwrap_or(false)
        }) {
          state.write().selected_session = None;
        }
      }
      state.sessions().track_field();
      state.sessions()
    },
  );
  // Effect::new(move |old_suggestions| {
  //   if let Some(old_suggestions) = old_suggestions {
  //     if state.selected_session().with_untracked(|q| {
  //       q.map(|f| f.selected_suggestion.map(|ss| f.suggestions.contains(&*ss.read()).not()))
  //         .flatten()
  //         .unwrap_or(false)
  //     }) {
  //       state.write().selected_session = None;
  //     }
  //   }
  //   state.selected_session().track();
  //   state.selected_session().map(|f| f.suggestions())
  // });
  let is_sidebar_visible = RwSignal::new(true);
  let toggle_sidebar = move |_| is_sidebar_visible.update(|f| *f = !*f);
  let add_session = move |_| {
    state.sessions().write().push(Session {
      // selected_suggestion: None,
      date_created: Local::now(),
      suggestions: Vec::new(),
      title: "جلسه جدید".to_string(),
    });
  };

  view! {
    <div id="app">
      <aside class="sidebar" class:open=is_sidebar_visible>
        <ul id="sessions">
          <ForEnumerate each=move || state.sessions() key=|item| item.date_created().get() let(index, session)>
            <li
              class:selected=move || state.selected_session().is_some_and(|f| *f.read() == *session.read())
              class="item"
              on:click=move |event: MouseEvent| {
                event.stop_propagation();
                state.write().selected_session = Some(session.into());
              }
              on:mousedown=move |event: MouseEvent| {
                event.stop_propagation();
                if event.which() == 3 {
                  session.title().set("HEEEEEELLO!".to_string());
                }
              }
            >
              {move || format!("{}\n{}", session.title().get(), session.date_created().get().to_string())}
            </li>
          </ForEnumerate>
        </ul>
      </aside>
      <button on:click=toggle_sidebar id="humbugger_button">
        =
      </button>
      <button on:click=add_session id="new_session_button">
        +
      </button>
      <main class="main">
        {move || match state.selected_session() {
          Some(selected_session) => {
            Either::Right(
              view! {
                <div class="session">
                  <div id="map"></div>
                  <ol class="suggestions">
                    {move || {
                      selected_session
                        .suggestions()
                        .iter_unkeyed()
                        .enumerate()
                        .map(|(index, suggestion)| {
                          view! { <SuggestionItem suggestion index /> }
                        })
                        .collect_view()
                    }}
                  </ol>
                  <div class="bottom_bar" class:open=is_sidebar_visible>
                    <textarea class="prompt" name="prompt" bind:value=state.prompt_text() />
                    <button class="send" on:click=answer>
                      ">"
                    </button>
                  </div>
                </div>
              },
            )
          }
          None => Either::Left(().into_view()),
        }}
      </main>
    </div>
  }
}

// #[component]
// fn SuggestionTabBar(#[prop(into)] session: Field<Session>) -> impl IntoView {
//   view! {
//     <div>
//       {move || {
//         session
//           .suggestions()
//           .iter_unkeyed()
//           .enumerate()
//           .map(|(index, suggestion)| {
//             view! {
//               <p
//                 style="padding:10px; background-color: #rgb(235, 250, 148);"
//                 on:click=move |_| session.write().selected_suggestion = Some(suggestion.into())
//               >

//                 {move || format!("پیشتنهاد {}", index + 1)}
//               </p>
//             }
//           })
//           .collect_view()
//       }}
//     </div>
//   }
// }

#[component]
fn SuggestionItem(#[prop(into)] suggestion: Field<Suggestion>, index: usize) -> impl IntoView {
  view! {
    <li class="item">
      // <div class="c-stepper__content">
      // <For each=move||suggestion.places().into_iter() key=move|place|place.get().clone() let:place>
      // <PlaceCard place=place.get() />
      // </For>
      <div class="options">
        <div class="step_number">{index}</div>
        <button class="next_suggestion">">"</button>
        <button class="previous_suggestion">"<"</button>
      </div>
      <PlaceCard place=suggestion.selected_place() {..} class="card" />
    // </div>
    </li>
  }
}

#[component]
fn PlaceCard(#[prop(into)] place: Field<Place>) -> impl IntoView {
  let place = place.get();
  view! {
    <div>
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

async fn ask_ai(prompt: String) -> Vec<Suggestion> {
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
  let mut neshan_history = serde_json::from_str::<NeshanDataModel>(include_str!(
    "taged_items/neshan_history_results_unique_with_tags.json"
  ))
  .unwrap();
  let mut neshan_museum = serde_json::from_str::<NeshanDataModel>(include_str!(
    "taged_items/neshan_museum_results_unique_with_tags.json"
  ))
  .unwrap();
  let mut neshan_restaurant = serde_json::from_str::<NeshanDataModel>(include_str!(
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
