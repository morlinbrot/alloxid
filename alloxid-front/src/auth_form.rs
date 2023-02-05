#![allow(non_snake_case)]
use dioxus::{core::to_owned, events::MouseEvent, prelude::*};

use crate::{JsonBody, UserCreatedData};

#[derive(PartialEq)]
pub enum AuthMode {
    In,
    Up,
}

impl AuthMode {
    fn other(&self) -> Self {
        match self {
            AuthMode::In => AuthMode::Up,
            AuthMode::Up => AuthMode::In,
        }
    }
}

impl std::fmt::Display for AuthMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMode::In => write!(f, "Sign In"),
            AuthMode::Up => write!(f, "Sign Up"),
        }
    }
}

#[derive(Props, PartialEq)]
pub struct Props<'a> {
    mode: AuthMode,
    user_id: &'a UseState<Option<String>>,
}

pub fn AuthForm<'a>(cx: Scope<'a, Props<'a>>) -> Element {
    let mode = &cx.props.mode;
    let other = mode.other();

    let _user_id = &cx.props.user_id;
    // let count = use_state(&cx, || 0);

    let create_user = move |_evt: MouseEvent| {
        // user_id.set(Some("id".to_string()));
        // use_coroutine(&cx, |_: UnboundedReceiver<()>| {
        //     to_owned![count];
        //     async_move {
        //         //
        //     }
        // });
        cx.spawn(async move {
            let json = serde_json::json!({
                "username": "synul",
                "password": "my-pw",
            });

            let res = reqwest::Client::new()
                .post("{API_URL}/user")
                .json(&json)
                .send()
                .await;

            match res {
                Ok(res) => {
                    let status = res.status();
                    let location = res.headers().get("Location").map(|h| h.to_owned());

                    let body: JsonBody<UserCreatedData> = res.json().await.unwrap();
                    let UserCreatedData { id, token } = body.data;

                    log::info!("User created: {:?}", (status, location, id, token));
                }

                Err(_err) => {
                    println!("Failed to create user - is the backend running?");
                }
            }
        });
    };

    let (subtext, suburl) = match mode {
        AuthMode::In => ("Don't have an account?", "/sign-up"),
        AuthMode::Up => ("Have an account?", "/sign-in"),
    };

    cx.render(rsx!(
        h2 { "{mode}" }
        ul {
            button { onclick: create_user, "Create user" }
            // button { onclick: get_user, "Get user" }
            // button { onclick: delete_user, "Delete user" }
        }
        form {
            // onsubmit: create_user,
            prevent_default: "onsubmit",

            label {
                // for: "username",
                "Username",
                input { r#type: "text", id: "username", name: "username", placeholder: "Username" }
            }

            label {
                // for: "password",
                "Password",
                input { r#type: "password", id: "password", name: "password", placeholder: "Password" }
            }

            // button { r#type: "submit", value: "Submit", "{mode}" }
        }

        small { "{subtext} " Link { to: suburl, "{other}" } }
    ))
}
