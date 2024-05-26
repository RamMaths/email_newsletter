use crate::domain::SubscriberEmail;
use lettre::message::{Mailbox, MultiPart, SinglePart};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TestResponse {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub text: String,
}

#[derive(Debug)]
pub struct EmailClient {
    pub host_url: String,
    pub from: SubscriberEmail,
    pub username: String,
    pub password: Secret<String>,
}

#[derive(Debug)]
pub enum EmailClientError {
    EmailBuilderError(lettre::error::Error),
    OpenRemoteConnectionError(lettre::transport::smtp::Error),
}

impl std::error::Error for EmailClientError {}

impl std::fmt::Display for EmailClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to send the email")
    }
}

impl From<lettre::error::Error> for EmailClientError {
    fn from(e: lettre::error::Error) -> Self {
        Self::EmailBuilderError(e)
    }
}

impl From<lettre::transport::smtp::Error> for EmailClientError {
    fn from(e: lettre::transport::smtp::Error) -> Self {
        Self::OpenRemoteConnectionError(e)
    }
}

impl EmailClient {
    pub fn new(
        host_url: String,
        from: SubscriberEmail,
        username: String,
        password: Secret<String>,
    ) -> Self {
        Self {
            host_url,
            from,
            username,
            password,
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), EmailClientError> {
        //Defining the email
        let email = Message::builder()
            .from(
                self.from
                    .as_ref()
                    .parse::<Mailbox>()
                    .expect("Could not parse the given from email to Mailbox"),
            )
            .to(recipient
                .as_ref()
                .parse::<Mailbox>()
                .expect("Could not parse the given to email to Mailbox"))
            .subject(subject)
            .multipart(
                MultiPart::mixed()
                    .singlepart(SinglePart::html(html_content.to_string()))
                    .singlepart(SinglePart::plain(text_content.to_string())),
            )?;

        // setting SMTP client credentials
        let creds = Credentials::new(
            self.username.to_owned(),
            self.password.expose_secret().to_owned(),
        );

        //Openning a remote connection to the SMTP server
        let mailer = SmtpTransport::starttls_relay(&self.host_url)?
            .credentials(creds)
            .build();

        mailer.send(&email)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::EmailClient;
    use crate::domain::SubscriberEmail;
    use secrecy::Secret;

    #[tokio::test]
    async fn sending_email_through_smtp() {
        let email_client = EmailClient::new(
            "sandbox.smtp.mailtrap.io".to_string(),
            SubscriberEmail::parse("ram.hdzven@gmail.com".to_string()).unwrap(),
            "cc16782b5fa486".to_string(),
            Secret::new("926b5352acd1f3".to_string()),
        );

        email_client
            .send_email(
                SubscriberEmail::parse("ram.hdzven@gmail.com".to_string())
                    .expect("Couldn't parse the email"),
                "Hello world",
                "Hello as well",
                "<h2>Hello from html</h2>",
            )
            .await
            .unwrap();
    }
}
