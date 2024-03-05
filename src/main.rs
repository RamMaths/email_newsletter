use email_newsletter::{
    startup::run,
    configuration::get_configuration
};
use std::net::TcpListener;
use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_log::LogTracer;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    //Redirect all log's events to our subscriber
    LogTracer::init().expect("Failed to set logger");
    //We are falling back to printing all spans at info-level or above
    //if the RUST_LOG envrionment variable has not been set.
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "email_newsletter".into(),
        // Output the formatted spans to stdout
        std::io::stdout
    );
    //The `with` method is provided by SubscriberExt, an extension
    //trait for Subscriber exposed by tracing_subscriber
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    // set_global_default can be used by applications to specify
    // what subscriber should be used to process spans
    set_global_default(subscriber).expect("Failed to set subscriber");
    //
    let configuration = get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address).expect("Failed to bind the address");
    let db_pool = PgPool::connect(
            &configuration.database.connection_string()
        )
        .await
        .expect("Failed to connect to postgres");
    run(listener, db_pool)?.await
}
