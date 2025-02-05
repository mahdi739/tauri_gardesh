use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use reactive_stores::{Field, StoreFieldIterator};

use crate::{components::suggestion_item::SuggestionItem, Session, SessionStoreFields};

#[component]
pub fn Suggestions(#[prop(into)] session: Field<Session>) -> impl IntoView {
  // Effect::new(move |_| {
  //   console_log(&format!("{:#?}", session.suggestions().get()));
  // });

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
