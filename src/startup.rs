use actix_web::{ 
    web,
    App, 
    HttpServer, 
    dev::Server
};
use std::net::TcpListener;
use super::routes::health_check;
use super::routes::subscribe;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
        .run();

    Ok(server)
}
