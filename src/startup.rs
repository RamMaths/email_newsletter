use actix_web::{ 
    web,
    App, 
    HttpServer, 
    dev::Server
};
use sqlx::PgPool;
use std::net::TcpListener;
use super::routes::health_check;
use super::routes::subscribe;
use super::email_client::EmailClient;
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient
    ) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(connection.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
