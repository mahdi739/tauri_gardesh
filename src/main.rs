use leptos::prelude::*;
use tauri_gardesh_ui::components::app::App;

fn main() {
  console_error_panic_hook::set_once();
  mount_to_body(App)
}
