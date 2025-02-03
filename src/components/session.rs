use leptos::{prelude::*, task::spawn_local};
use reactive_stores::Field;

use crate::{ask_ai, components::suggesions::Suggestions, Session, State, StateStoreFields};

#[component]
pub fn SessionContent(
  #[prop(into)] state: Field<State>,
  #[prop(into)] session: Field<Session>,
) -> impl IntoView {
  let answer = move |_| {
    spawn_local(async move {
      state.answering().set(true);
      let answer = ask_ai(state.prompt_text().get()).await;
      state.answering().set(false);
      //   console_log(&format!("{:#?}", answer.clone()));
      session.update(|ss| {
        ss.suggestions = answer;
      });
    });
  };
  view! {
    <div>
      <div id="map"></div>
      <Suggestions session {..} class="suggestions" />
      <div class="bottom_bar">
        <textarea
          class="prompt"
          name="prompt"
          bind:value=state.prompt_text()
          class:open=move || state.is_sidebar_visible().get()
        />
        <button
          class="fa fa-send send"
          on:click=answer
          disabled=move || state.answering().get()
        ></button>
      </div>
    </div>
  }
}
