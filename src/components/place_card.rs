use leptos::prelude::*;
use reactive_stores::Field;

use crate::Place;

#[component]
pub fn PlaceCard(#[prop(into)] place: Field<Place>) -> impl IntoView {
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
