use crate::configure_app;
use crate::settings::Settings;

pub struct TestApp {
    pub address: String,
}

pub async fn spawn_test_app() -> TestApp {
    let Settings { app, .. } = Settings::new().expect("Failed to load configuration.");

    let address = format!("http://{}:{}", app.host, app.port);
    let app = configure_app().await.unwrap();

    let _ = async_std::task::spawn(app.listen(address.clone()));
    TestApp { address }
}
