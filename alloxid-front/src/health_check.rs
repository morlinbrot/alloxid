#![allow(non_snake_case)]
use dioxus::prelude::*;

pub fn HealthCheck(cx: Scope) -> Element {
    let message = use_state(&cx, || "".to_string());

    let fut = use_future(&cx, (), |_| async move {
        let client = reqwest::Client::new();
        client
            .get("http://localhost:3000/health-check")
            .send()
            .await?
            .text()
            .await
    });

    match fut.value() {
        Some(Ok(msg)) => {
            message.set(msg.to_string());
        }
        Some(Err(err)) => {
            log::error!("ERR: {}", err);
            message.set("There was an error. Is the backend running?".to_string());
        }
        None => {
            message.set("Fetching...".to_string());
        }
    };

    cx.render(rsx!(
        div {
            h2 { "Health check" }
            p { "Server says:" br { } "{message}" }
        }
    ))
}
