use email_newsletter::{
    configuration::get_configuration,
    telemetry::*,
    startup::*
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    //setting the subscriber (telemetry)
    let subscriber = get_subscriber("email_newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration()
        .expect("Failed to load configuration file");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;

    Ok(())
}

