#![allow(unused)]

use better_default::Default;
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
  #[default(true)]
  is_sidebar_visible: bool,
  answering: bool,
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
  let state = Store::new(State::default());
  let selected_session: RwSignal<Option<Field<Session>>> = RwSignal::new(None);
  let map_options = Object::new();
  config_map(&map_options);
  let markers = StoredValue::new_local(Vec::<Marker>::new());
  let map_ref: RwSignal<Option<Map>, LocalStorage> = RwSignal::new_local(None);
  Effect::new(move |_| {
    if let (Some(selected_session), Some(map_ref)) = (selected_session.get(), map_ref.get()) {
      markers.get_value().iter().for_each(Marker::remove);
      markers.set_value(
        selected_session
          .suggestions()
          .iter_unkeyed()
          .map(|sg| {
            let (x, y) = sg.selected_place().with(|f| (f.location.x, f.location.y));
            Marker::newMarker()
              .setLngLat(&JsValue::from(Array::of2(&JsValue::from_f64(x), &JsValue::from_f64(y))))
          })
          .inspect(|marker| {
            marker.addTo(&map_ref);
          })
          .collect_vec(),
      );
    }
  });

  let answer = move |ev: MouseEvent| {
    spawn_local(async move {
      state.answering().set(true);
      let answer = ask_ai(state.prompt_text().get()).await;
      state.answering().set(false);
      console_log(&format!("{:#?}", answer.clone()));
      selected_session.update(|f| {
        f.map(|ss| ss.write().suggestions = answer);
      });
    });
  };

  // let r = format!("{:#?}", state.path().into_iter().collect_vec());
  // state.read().selected_session.unwrap().date_created()
  // state.sessions().at_unkeyed(0).reader()
  // console_log(&"About to rendering view");
  // let r = AtKeyed::new(state.sessions(), state.sessions()..get()[0].date_created);

  // state.sessions().at_unkeyed(0).date_created().get()
  Effect::new(move |_| {
    let map_options = map_options.clone();
    request_animation_frame(move || {
      // console_log(&format!("Checking...\n {:#?},\n{:#?}",map_ref.get().map(|m|m.obj),selected_session.get().get()));
      if map_ref.read().is_none() && selected_session.read().is_some() {
        console_log("Setting...");
        map_ref.set(Some(Map::newMap(&JsValue::from(map_options))));
      } else if selected_session.read().is_none() {
        map_ref.set(None);
      }
    });

    selected_session.track();
  });
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
  // let is_sidebar_visible = RwSignal::new(true);
  let toggle_sidebar = move |_| state.is_sidebar_visible().update(|f| *f = !*f);
  let add_session = move |_| {
    state.sessions().write().insert(
      0,
      Session {
        // selected_suggestion: None,
        date_created: Local::now(),
        suggestions: Vec::new(),
        title: "جلسه ".to_string(),
      },
    );
    selected_session.set(state.sessions().into_iter().next().map(Into::into));
  };

  state.sessions().write().push(Session {
    date_created: Local::now(),
    suggestions: Vec::new(),
    title: "جلسه ".to_string(),
  });
  selected_session.set(state.sessions().into_iter().next().map(Into::into));

  view! {
    <div id="app">
      <aside class="sidebar" class:open=move || state.is_sidebar_visible().get()>
        <ul class="sessions">
          <ForEnumerate each=move || state.sessions() key=|item| item.date_created().get() let(index, session)>
            <li
              class:selected=move || selected_session.read().is_some_and(|f| *f.read() == *session.read())
              class="item"
              on:click=move |event: MouseEvent| {
                event.stop_propagation();
                selected_session.set(Some(session.into()));
              }
              on:mousedown=move |event: MouseEvent| {
                event.stop_propagation();
                if event.which() == 3 {
                  session.title().set("HEEEEEELLO!".to_string());
                }
              }
            >
              <button
                on:click=move |_| {
                  let selected_session_value = selected_session.get().map(|f| f.get());
                  let session_value = session.get();
                  state
                    .sessions()
                    .update(|s| {
                      s.remove(index.get());
                    });
                  if selected_session_value.is_some_and(|f| f == session_value) {
                    selected_session.set(None);
                    selected_session.set(state.sessions().into_iter().next().map(Into::into));
                  }
                }
                class="fa fa-trash delete"
              ></button>
              {move || format!("{}\n{}", session.title().get(), session.date_created().get().format("%d/%m/%Y %H:%M"))}

            </li>
          </ForEnumerate>
        </ul>
      </aside>
      <button on:click=toggle_sidebar id="humbugger_button" class="fa fa-bars" />
      <button
        on:click=add_session
        id="new_session_button"
        class="fa fa-plus"
        class:open=move || state.is_sidebar_visible().get()
      >
        <span>{move || state.is_sidebar_visible().get().then_some(" چت جدید").unwrap_or("")}</span>
      </button>

      <main class="main">
        {move || match selected_session.get() {
          Some(selected_session) => {
            Either::Right(
              view! {
                <div class="session">
                  <div id="map"></div>
                  <Suggestions selected_session {..} class="suggestions" />
                  <div class="bottom_bar">
                    <textarea
                      class="prompt"
                      name="prompt"
                      bind:value=state.prompt_text()
                      class:open=move || state.is_sidebar_visible().get()
                    />
                    <button class="fa fa-send send" on:click=answer disabled=move || state.answering().get()></button>
                  </div>
                </div>
              },
            )
          }
          None => Either::Left(()),
        }}
      </main>
    </div>
  }
}

#[component]
fn Suggestions(#[prop(into)] selected_session: Field<Session>) -> impl IntoView {
  view! {
    <ol>
      {move || {
        selected_session
          .suggestions()
          .iter_unkeyed()
          .enumerate()
          .map(|(index, suggestion)| {

            view! { <SuggestionItem suggestion index {..} class="item" /> }
          })
          .collect_view()
      }}
    </ol>
  }
}

#[component]
fn SuggestionItem(#[prop(into)] suggestion: Field<Suggestion>, index: usize) -> impl IntoView {
  view! {
    <li>
      <div class="options">
        <div class="step_number">{index + 1}</div>
        {move || {
          (suggestion.places().iter_unkeyed().count() > 1)
            .then_some(
              view! {
                <button on:click=move |_| suggestion.next() class="next_suggestion fa fa-angle-right" />
                <button on:click=move |_| suggestion.prev() class="previous_suggestion fa fa-angle-left" />
              },
            )
        }}

      </div>
      <PlaceCard place=suggestion.selected_place() {..} class="card" />
    </li>
  }
}

#[component]
fn PlaceCard(#[prop(into)] place: Field<Place>) -> impl IntoView {
  // let place = place.get();
  view! {
    <div>
      <h2>{move || place.read().title.clone()}</h2>
      <p>
        <strong>"Category:"</strong>
        {move || place.read().category.clone()}
      </p>
      <p>
        <strong>"Type:"</strong>
        {move || place.read().r#type.to_string()}
      </p>
      <p>
        <strong>"Region:"</strong>
        {move || place.read().region.clone()}
      </p>
      <p>
        <strong>"Neighbourhood:"</strong>
        {move || place.read().neighbourhood.clone()}
      </p>
      <p>
        <strong>"Location:"</strong>
        {move || format!("{}, {}", place.read().location.x, place.read().location.y)}
      </p>
      <p>
        <strong>Tags:</strong>
        {move || place.read().tags.join(", ")}
      </p>
    </div>
  }
}

async fn ask_ai(prompt: String) -> Vec<Suggestion> {
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
