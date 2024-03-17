use email_newsletter::{
    startup::run,
    configuration::get_configuration
};
use std::net::TcpListener;
use sqlx::PgPool;
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
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address).expect("Failed to bind the address");
    let db_pool = PgPool::connect(
            &configuration.database.connection_string()
        )
        .await
        .expect("Failed to connect to postgres");
    run(listener, db_pool)?.await
}
