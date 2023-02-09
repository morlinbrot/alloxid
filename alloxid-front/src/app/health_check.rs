use leptos::*;

async fn health_check(_: ()) -> Option<String> {
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

async fn health_check2() -> Option<String> {
    let client = reqwest::Client::new();
    log!("FETCHING...");
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
    let message = create_resource(cx, || (), health_check);

    let fetch_message = create_action(cx, |_: &()| health_check2());
    let handle_click = move |_| fetch_message.dispatch(());

    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        cx,
        <div>
            <button on:click=on_click>"Click Me: " {count}</button>

            <h4>"Health click:"</h4>
            <button on:click=handle_click>"Fetch message"</button>
            <p>"Message: "{fetch_message.value()}</p>

            <h4>"Health check:"</h4>
            <Suspense fallback=|| view! { cx, "Loading..." }>
                {move || message.read().map(|msg| match msg {
                    None => view! { cx,  <div>"Error fetching the message. Is the backend running?"</div> },
                    Some(msg) => view! { cx, <div>{msg}</div> }
                })}
            </Suspense>
        </div>
    }
}
