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
    cx.render(rsx! {
        Router {
            header { class: "container", style: "padding-top: 3rem; padding-bottom: 3rem;",
                nav {
                    ul {
                        li {
                            h1 { style: "margin-bottom: 0;",  "alloxid" }
                            strong {  "🥳 'tis all oxidized! 🎉" }
                        }
                    }

                    ul {
                        li { Link { to: "/health", "health check" } }
                        li { Link { to: "/sign-in", button { "Sign In" } } }
                    }
                }
            }

            main { class: "container",
                Route { to: "/",
                    h2 { "Home" }
                }
                Route { to: "/health",
                    HealthCheck { }
                }
                Route { to: "",
                    h2 { "Oops, 404." }
                }
            }
        }
    })
}
