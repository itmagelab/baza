use baza_web::App;
use leptos::prelude::*;

pub fn main() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Hello from Rust".into());
    tracing_wasm::set_as_global_default();

    mount_to_body(|| {
        view! { <App/> }
    })
}
