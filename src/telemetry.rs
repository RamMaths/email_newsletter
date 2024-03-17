use tracing::subscriber;
use tracing_subscriber::EnvFilter;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink
) -> impl subscriber::Subscriber + Send + Sync 
where 
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static 
{
    //We are falling back to printing all spans at info-level or above
    //if the RUST_LOG envrionment variable has not been set.
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(
        name,
        // Output the formatted spans to stdout
        sink
    );
    //The `with` method is provided by SubscriberExt, an extension
    //trait for Subscriber exposed by tracing_subscriber
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl subscriber::Subscriber + Send + Sync) {
    //Redirect all log's events to our subscriber
    LogTracer::init().expect("Failed to set logger");
    // set_global_default can be used by applications to specify
    // what subscriber should be used to process spans
    subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
