use actix_web::{ 
    web,
    App, 
    HttpServer, 
    dev::Server
};
use sqlx::PgPool;
use std::net::TcpListener;
use super::email_client::EmailClient;
use tracing_actix_web::TracingLogger;
use super::configuration::{
    Settings,
    DatabaseSettings
};
use sqlx::postgres::PgPoolOptions;
use super::routes::{
    health_check,
    subscribe,
    confirm
};

pub struct Application {
    port: u16,
    server: Server
}
pub struct ApplicationBaseUrl(pub String);

impl Application {
    fn run(
        listener: TcpListener,
        connection: PgPool,
        email_client: EmailClient,
        base_url: String
        ) -> Result<Server, std::io::Error> {

        let connection = web::Data::new(connection);
        let email_client = web::Data::new(email_client);
        let base_url = web::Data::new(ApplicationBaseUrl(base_url));

        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/health_check", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
                .route("/subscriptions/confirm", web::get().to(confirm))
                .app_data(connection.clone())
                .app_data(email_client.clone())
                .app_data(base_url.clone())
        })
        .listen(listener)?
        .run();

        Ok(server)
    }

    pub async fn build(configuration: Settings) -> Result<Application, std::io::Error> {
        //Database
        let db_pool = get_connection_pool(&configuration.database);

        //Building an email client
        let email_sender = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");

        let email_client = EmailClient::new(
            configuration.email_client.host_url.to_owned(),
            email_sender,
            configuration.email_client.username.to_owned(),
            configuration.email_client.password
        );

        let address = format!(
            "{}:{}",
            &configuration.application.host,
            &configuration.application.port
        );
        let listener = TcpListener::bind(&address).expect("Failed to bind the address");

        let port = listener.local_addr().unwrap().port();
        let server = Application::run(listener, db_pool, email_client, configuration.application.base_url)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}


pub fn get_connection_pool (
    configuration: &DatabaseSettings
) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.with_db())
}
