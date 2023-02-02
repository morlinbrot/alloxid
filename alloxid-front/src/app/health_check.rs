use leptos::*;

async fn health_check(_how_many: i32) -> Option<String> {
    let client = reqwest::Client::new();
    client
        .get("http://localhost:3000/health-check")
        .send()
        .await
        .map_err(|e| log::error!("{e}"))
        .ok()?
        .text()
        .await
        .ok()
}

#[component]
pub fn HealthCheck(cx: Scope) -> impl IntoView {
    let (how_many, set_how_many) = create_signal(cx, 1);

    let message = create_resource(cx, how_many, health_check);
    // let _: () = message;

    view! {
        cx,
        <div>
            <p>"Health check:"</p>
            <p>
                { move || message.read().map(|msg| match msg {
                    None => format!("Failed to fetch message. Is the backend running?"),
                    Some(msg) => format!("{msg}"),
                })}
            </p>
        </div>
    }
}
