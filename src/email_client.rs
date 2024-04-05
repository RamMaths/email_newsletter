use crate::domain::SubscriberEmail;
use reqwest::{
    Client,
    Url
};
use secrecy::{
    Secret,
    ExposeSecret
};
use serde_json::json;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: std::time::Duration
    ) -> Self {
        Self {
            http_client: Client::builder()
                .timeout(timeout)
                .build()
                .unwrap(),
            base_url,
            sender,
            authorization_token
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = Url::parse(&self.base_url)?;

        let request_body = json!({
            "from": {"email": self.sender.as_ref()},
            "to": [{"email": recipient.as_ref()}],
            "subject": subject,
            "attachments": [
                {
                    "content": html_content,
                    "type": "text/html"
                }
            ],
            "text": text_content,
            "json": true
        });

        self
            .http_client.post(url)
            .header("Api-Token", self.authorization_token.expose_secret())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence}; use fake::{Fake, Faker};
    use wiremock::matchers::{
        any,
        header,
        method,
        header_exists
    };
    use wiremock::{
        Mock, 
        MockServer, 
        ResponseTemplate
    };
    use secrecy::Secret;
    use claims::{
        assert_ok,
        assert_err
    };

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(base_url, email(), Secret::new(Faker.fake()), std::time::Duration::from_millis(200))
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("Api-Token"))
            .and(header("Content-Type", "application/json"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        //Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        //Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        //Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        //Assert
        assert_err!(outcome);
    }
}
