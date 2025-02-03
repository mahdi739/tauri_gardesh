use leptos::prelude::*;
use reactive_stores::{Field, StoreFieldIterator};

use crate::{components::place_card::PlaceCard, Suggestion, SuggestionExt, SuggestionStoreFields};

#[component]
pub fn SuggestionItem(#[prop(into)] suggestion: Field<Suggestion>, index: usize) -> impl IntoView {
  view! {
    <li>
      <div class="options">
        <div class="step_number">{index + 1}</div>
        <Show when=move || { suggestion.places().iter_unkeyed().count() > 1 }>
          <button on:click=move |_| suggestion.next() class="next_suggestion fa fa-angle-right" />
          <button
            on:click=move |_| suggestion.prev()
            class="previous_suggestion fa fa-angle-left"
          />
        </Show>
      </div>
      <PlaceCard place=suggestion.selected_place() {..} class="card" />
    </li>
  }
}
