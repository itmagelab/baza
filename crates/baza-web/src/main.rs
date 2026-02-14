use baza_web::App;

pub fn main() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Hello from Rust".into());
    tracing_wasm::set_as_global_default();

    yew::Renderer::<App>::new().render();
}
