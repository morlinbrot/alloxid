// use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
// use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_log::LogTracer;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

use crate::StateExtension;

pub struct LogInfo {
    app_port: usize,
    req_id: Uuid,
    db_name: String,
}

impl LogInfo {
    pub fn from_req(state: StateExtension) -> Self {
        let req_id = Uuid::new_v4();
        Self {
            app_port: state.settings.app.port,
            req_id,
            db_name: state.settings.database.name.clone(),
        }
    }
}

impl std::fmt::Display for LogInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LogInfo{{app_port={} req_id={} db_name={}}}",
            self.app_port, self.req_id, self.db_name
        )
    }
}

pub fn get_subscriber(_name: String, env_filter: String) -> impl tracing::Subscriber + Send + Sync {
    // Set the default log level.
    // Overwrite with something like
    // RUST_LOG="debug,tide=warn,sqlx=warn,surf=warn,isahc=off"
    // for debugging.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    // We configure a logger that resembles the default env_logger.
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .finish();

    // let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    // let subscriber = Registry::default()
    //     .with(env_filter)
    //     .with(JsonStorageLayer)
    //     .with(formatting_layer);

    subscriber
}

pub fn init_subscriber(subscriber: impl tracing::Subscriber + Send + Sync) {
    // Switch on/off to enable logs from all other crates.
    LogTracer::init().expect("Failed to set logger.");

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
