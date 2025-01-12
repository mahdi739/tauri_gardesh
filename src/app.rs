#![allow(unused)]

use dotenvy_macro::dotenv;
use leptos::leptos_dom::logging::console_log;
use leptos::tachys::html::property::IntoProperty;
use reactive_stores::{OptionStoreExt as _, Store, StoreFieldIterator};
use serde_json::to_value;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;
use std::str::FromStr;
use std::{
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

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
  async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct AnswerArgs {
  search_text: String,
}
const EPSILON: f64 = 1e-10;
#[derive(Serialize, Deserialize)]
struct GreetArgs {
  name: String,
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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct ResponseModel {
  names: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Place {
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
        .collect::<Vec<_>>(),
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
  let mut filtered_places: Vec<_> = places
    .into_iter()
    .filter(|place| selected_types.contains(&PlaceType::from_str(&place.r#type).unwrap()))
    .collect();

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
  selected_places.iter().map(|f| f.title.clone()).collect::<Vec<String>>()
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
          .collect::<Vec<_>>(),
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
      let args = serde_wasm_bindgen::to_value(&GreetArgs { name: prompt_text.get() }).unwrap();
      // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
      let new_msg =
        serde_wasm_bindgen::from_value::<Vec<Place>>(invoke("greet", args).await).unwrap();
      console_log(&format!("{new_msg:#?}"));
      places.set(new_msg);
    });
  };
  console_log(&"About to rendering view");
  view! {
    <div>
      <div id="map" style="height:500px;width:100%;"></div>

      <label>"Number of Places: "</label>
      <label>"Number of Places: "</label>
      <input type="number" bind:value=state.num_places() />
      <label>"Budget: "</label>
      <input type="number" bind:value=state.budget() />

      <label>"Time: "</label>
      <input type="number" bind:value=state.time() />
      <label>"Options: "</label>
      <div id="checkboxes">
        <For each=move || state.select_places() key=move |f| f.clone().place_type().get() let:item>
          <label>
            <input type="checkbox" bind:checked=item.clone().checked() />
            {move || item.place_type().get().to_string()}
          </label>
        </For>

      </div>
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
      <ol class="c-stepper">
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
        {place.r#type}
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
