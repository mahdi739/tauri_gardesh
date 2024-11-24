#![allow(unused)]

use enum_all_variants::AllVariants;
use leptos::leptos_dom::logging::console_log;
use rand::seq::SliceRandom;
use reactive_stores::{OptionStoreExt as _, Store, StoreFieldIterator};
use serde_json::to_value;
use std::collections::HashSet;
use std::fmt::Debug;
use std::{
  fs::read_to_string,
  hash::{Hash, Hasher},
  ops::Not,
};
use wasm_bindgen::prelude::*;

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use leptos::{either::Either, prelude::*, task::spawn_local};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, AllVariants)]
enum PlaceType {
  Museum,
  Park,
  ThemePark,
  FineDining,
  FastFood,
  Cafe,
  StreetFood,
  Monument,
  Theater,
  ConcertHall,
  Cinema,
  AmusementPark,
  Mall,
  Market,
  SouvenirShop,
  Mosque,
}
impl PlaceType {
  pub fn from_str(name: impl AsRef<str>) -> PlaceType {
    match name.as_ref() {
      "موزه" => PlaceType::Museum,
      "پارک" => PlaceType::Park,
      "پارک تفریحی" => PlaceType::ThemePark,
      "رستوران عالی" => PlaceType::FineDining,
      "فست‌فود" => PlaceType::FastFood,
      "کافه" => PlaceType::Cafe,
      "غذای خیابانی" => PlaceType::StreetFood,
      "بنای تاریخی" => PlaceType::Monument,
      "تئاتر" => PlaceType::Theater,
      "سالن کنسرت" => PlaceType::ConcertHall,
      "سینما" => PlaceType::Cinema,
      "شهربازی" => PlaceType::AmusementPark,
      "مرکز خرید" => PlaceType::Mall,
      "بازار" => PlaceType::Market,
      "فروشگاه سوغات" => PlaceType::SouvenirShop,
      "مسجد" => PlaceType::Mosque,
      _ => panic!("No Place Found"),
    }
  }
  pub const fn label(&self) -> &'static str {
    match self {
      PlaceType::Museum => "موزه",
      PlaceType::Park => "پارک",
      PlaceType::ThemePark => "پارک تفریحی",
      PlaceType::FineDining => "رستوران عالی",
      PlaceType::FastFood => "فست‌فود",
      PlaceType::Cafe => "کافه",
      PlaceType::StreetFood => "غذای خیابانی",
      PlaceType::Monument => "بنای تاریخی",
      PlaceType::Theater => "تئاتر",
      PlaceType::ConcertHall => "سالن کنسرت",
      PlaceType::Cinema => "سینما",
      PlaceType::AmusementPark => "شهربازی",
      PlaceType::Mall => "مرکز خرید",
      PlaceType::Market => "بازار",
      PlaceType::SouvenirShop => "فروشگاه سوغات",
      PlaceType::Mosque => "مسجد",
    }
  }
}
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
      select_places: PlaceType::all_variants()
        .iter()
        .map(|f| SelectPlace { checked: true, place_type: f.to_owned() })
        .collect::<Vec<_>>(),
      result: "".to_string(),
    }
  }
}

fn shuffle_vec<T: Debug>(vec: &mut Vec<T>) {
  // console_log(&format!("{vec:#?}"));
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
    .filter(|place| selected_types.contains(&PlaceType::from_str(&place.r#type)))
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

#[component]
pub fn App() -> impl IntoView {
  // let places = serde_json::from_str::<Vec<Place>>(include_str!("../data.json")).unwrap();
  let state = Store::new(State::default());
  let prompt_text = RwSignal::new("".to_string());
  let answer_text = RwSignal::new("".to_string());
  let t = RwSignal::new(String::new());
  let answer = move |ev: MouseEvent| {
    ev.prevent_default();

    log::info!("Tauri is awesome!");
    t.set("Hi From outside of the spaawn".to_string());
    console_log("Hi From outside of the spaawn");
    spawn_local(async move {
      console_log("Hi");
      // let args = to_value(&CounterArgs { count }).unwrap();
      t.set(format!(
        "{:#?}",
        serde_wasm_bindgen::to_value(&AnswerArgs { search_text: prompt_text.get() })
      ));
      let args = serde_wasm_bindgen::to_value(&GreetArgs { name: prompt_text.get() }).unwrap();
      // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
      let new_msg =
        serde_wasm_bindgen::from_value::<ResponseModel>(invoke("greet", args).await).unwrap();

      // let res = invoke(
      //   "answer",
      //   serde_wasm_bindgen::to_value(&AnswerArgs { search_text: prompt_text.get() }).unwrap(),
      // )
      // .await;
      let dataset = serde_json::from_str::<Model>(include_str!(
        "neshan_history_results_unique_with_tags_translated.json"
      ))
      .unwrap();
      let filtered =
        dataset.items.iter().filter(|f| new_msg.names.contains(&f.title)).collect::<Vec<_>>();

      t.set(format!("{:#?}", filtered.clone()));
      // console_log(&res);
      // answer_text.set(res);
    });
  };
  view! {
    <div>
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
            {move || item.place_type().get().label()}
          </label>
        </For>

      </div>
      <input type="text" name="prompt" id="prompt" bind:value=prompt_text />
      <button on:click=answer>"Generate Suggestion"</button>
      <p dir="auto">{move || t.get()}</p>
      {move || answer_text.get()}
    </div>
  }
}
// move |ev| {
//           console_log(&format!("SALAM"));
//           state.result().set(format!("{:#?}", select_places(&state.read(), places.clone())));
//         }
