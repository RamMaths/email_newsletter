use email_newsletter::startup::*;
use email_newsletter::{configuration::DatabaseSettings, telemetry::*};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute the request")
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        let url = format!("{}/newsletters", &self.address);
        reqwest::Client::new()
            .post(&url)
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

static TRAICING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRAICING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = email_newsletter::configuration::get_configuration()
            .expect("Failed to get the configuration file");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        // Use the mock server as email API
        c.email_client.host_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build the server");
    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", &application_port);
    let _ = tokio::spawn(application.run_until_stopped());

    //We return the application address to the caller
    TestApp {
        address,
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, &config.database_name).as_str())
        .await
        .expect("Failed to create database");

    //Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
