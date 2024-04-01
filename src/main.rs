use email_newsletter::{
    startup::run,
    configuration::get_configuration
};
use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use email_newsletter::telemetry::*;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber(
        "email_newsletter".to_string(),
        "info".to_string(),
        std::io::stdout
    );
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration");

    println!("Application attempting to run on {}:{}", &configuration.application.host, &configuration.application.port);

    let address = format!("{}:{}", &configuration.application.host, &configuration.application.port);
    let listener = TcpListener::bind(&address).expect("Failed to bind the address");
    let db_pool = PgPoolOptions::new()
        .connect_lazy_with(configuration.database.with_db());
    run(listener, db_pool)?.await
}
