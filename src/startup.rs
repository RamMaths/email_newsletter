use actix_web::{ 
    web,
    App, 
    HttpServer, 
    dev::Server,
    middleware::Logger
};
use sqlx::PgPool;
use std::net::TcpListener;
use super::routes::health_check;
use super::routes::subscribe;

pub fn run(
    listener: TcpListener,
    connection: PgPool
    ) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
