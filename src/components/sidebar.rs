use chrono::Local;
use leptos::prelude::*;
use reactive_stores::Field;
use web_sys::MouseEvent;

use crate::{Session, SessionStoreFields, State, StateStoreFields};

#[component]
pub fn Sidebar(
  #[prop(into)] state: Field<State>,
  #[prop(into)] selected_session: RwSignal<Option<Field<Session>>>,
) -> impl IntoView {
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

  view! {
    <aside>
      <ul class="sessions">
        <ForEnumerate
          each=move || state.sessions()
          key=|item| item.date_created().get()
          let(index,
          session)
        >
          <li
            class:selected=move || {
              selected_session.read().is_some_and(|f| *f.read() == *session.read())
            }
            class="item"
            on:click=move |event: MouseEvent| {
              event.stop_propagation();
              selected_session.set(Some(session.into()));
            }
          >
            <button
              on:click=move |_| {
                let selected_session_value = selected_session.get().map(|f| f.get());
                let session_value = session.get();
                state.sessions().write().remove(index.get());
                if selected_session_value.is_some_and(|f| f == session_value) {
                  selected_session.set(None);
                  selected_session.set(state.sessions().into_iter().next().map(Into::into));
                }
              }
              class="fa fa-trash delete"
            ></button>
            {move || {
              format!(
                "{}\n{}",
                session.title().get(),
                session.date_created().get().format("%d/%m/%Y %H:%M"),
              )
            }}

          </li>
        </ForEnumerate>
      </ul>
      <button on:click=toggle_sidebar id="humbugger_button" class="fa fa-bars" />
      <button
        on:click=add_session
        id="new_session_button"
        class="fa fa-plus"
        class:open=move || state.is_sidebar_visible().get()
      >
        <span>
          {move || { state.is_sidebar_visible().get().then_some(" چت جدید").unwrap_or("") }}
        </span>
      </button>
    </aside>
  }
}
