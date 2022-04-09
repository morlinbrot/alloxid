use dioxus::prelude::*;

mod health_check;
use health_check::*;

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    dioxus::web::launch(app);
}

fn app(cx: Scope) -> Element {
    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "🥳 alloxid-front 🎉" }
            HealthCheck { }
        }
    ))
}
