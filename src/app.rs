#![allow(unused)]

use enum_all_variants::AllVariants;
use leptos::leptos_dom::logging::console_log;
use rand::seq::SliceRandom;
use reactive_stores::{OptionStoreExt as _, Store, StoreFieldIterator};
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
use web_sys::{MouseEvent, SubmitEvent};

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
  async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
  name: &'a str,
}
const EPSILON: f64 = 1e-10;

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
struct Place {
  category: String,
  place_type: PlaceType,
  name: String,
  address: String,
  latitude: f64,
  longitude: f64,
  description: String,
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
  let mut filtered_places: Vec<_> =
    places.into_iter().filter(|place| selected_types.contains(&place.place_type)).collect();

  shuffle_vec(&mut filtered_places);

  let mut selected_places = Vec::new();
  let mut place_types = HashSet::new();

  for place in filtered_places {
    if selected_places.len().to_string() == state.num_places {
      break;
    }
    if !place_types.contains(&place.place_type) {
      place_types.insert(place.place_type.clone());
      selected_places.push(place);
    }
  }
  selected_places.iter().map(|f| f.name.clone()).collect::<Vec<String>>()
}

#[component]
pub fn App() -> impl IntoView {
  let places = serde_json::from_str::<Vec<Place>>(include_str!("../data.json")).unwrap();
  let state = Store::new(State::default());

  view! {
    <div>
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
      <input
        type="button"
        value="Generate Suggestion"
        on:click=move |ev| {
          console_log(&format!("SALAM"));
          state.result().set(format!("{:#?}", select_places(&state.read(), places.clone())));
        }
      />
      {move || state.result().get()}
    </div>
  }
}
