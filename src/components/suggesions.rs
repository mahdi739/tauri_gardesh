use leptos::prelude::*;
use reactive_stores::{Field, StoreFieldIterator};

use crate::{components::suggestion_item::SuggestionItem, Session, SessionStoreFields};
#[component]
pub fn Suggestions(#[prop(into)] session: Field<Session>) -> impl IntoView {
  view! {
    <ol>
      {move || {
        session
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
