use email_newsletter::{
    startup::run,
    configuration::get_configuration,
    telemetry::*,
    email_client::EmailClient
};
use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber(
        "email_newsletter".to_string(),
        "info".to_string(),
        std::io::stdout
    );
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration");

    let address = format!("{}:{}", &configuration.application.host, &configuration.application.port);
    let listener = TcpListener::bind(&address).expect("Failed to bind the address");

    //Database
    let db_pool = PgPoolOptions::new()
        .connect_lazy_with(configuration.database.with_db());

    //Building an email client
    let email_sender = configuration.email_client.sender().expect("Invalid sender email address");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url.clone(),
        email_sender,
        configuration.email_client.authorization_token,
        timeout
    );


    println!("Application running on {}:{}", &configuration.application.host, &configuration.application.port);
    //
    run(listener, db_pool, email_client)?.await
}
