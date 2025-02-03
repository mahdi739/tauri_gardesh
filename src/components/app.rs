use crate::components::session_content::SessionContent;
use crate::components::sidebar::Sidebar;

use crate::{
  Map, Marker, Session, SessionStoreFields, State, StateStoreFields, SuggestionStoreFields,
};
use chrono::Local;
use dotenvy_macro::dotenv;
use iter_tools::Itertools;
use leptos::leptos_dom::logging::console_log;
use leptos::{either::Either, prelude::*};
use reactive_stores::{Field, Store, StoreFieldIterator};
use wasm_bindgen::prelude::*;
use web_sys::js_sys::{Array, Object, Reflect};

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
  
  state.sessions().write().push(Session {
    date_created: Local::now(),
    suggestions: Vec::new(),
    title: "جلسه ".to_string(),
  });
  selected_session.set(state.sessions().into_iter().next().map(Into::into));

  view! {
    <div id="app">
      <Sidebar
        state
        selected_session
        {..}
        class="sidebar"
        class:open=move || state.is_sidebar_visible().get()
      />
      <main class="main">
        {move || match selected_session.get() {
          Some(selected_session) => {
            Either::Right(
              view! { <SessionContent state session={selected_session} {..} class="session" /> },
            )
          }
          None => Either::Left(()),
        }}
      </main>
    </div>
  }
}
