use email_newsletter::{
    startup::run,
    configuration::get_configuration
};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address).expect("Failed to bind the address");
    run(listener)?.await
}
