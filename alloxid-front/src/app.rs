use dioxus::prelude::*;

use crate::{
    auth_form::{AuthForm, AuthMode},
    health_check::HealthCheck,
};
// use fermi::use_read;

// use crate::USER_ID;

pub fn app(cx: Scope) -> Element {
    // let user_id = use_read(&cx, USER_ID);

    let user_id: &UseState<Option<String>> = use_state(&cx, || None);

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
                    AuthForm { mode: AuthMode::In, user_id: user_id }
                }
                Route { to: "/health",
                    HealthCheck { }
                }
                Route { to: "/sign-in",
                    // AuthForm { mode: AuthMode::In }
                }
                Route { to: "/sign-up",
                    // AuthForm { mode: AuthMode::Up }
                }
                Route { to: "",
                    h2 { "Oops, 404." }
                }
            }
        }
    })
}
