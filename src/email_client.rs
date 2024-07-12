use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde_json::json;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TestResponse {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub text: String,
}

#[derive(Debug)]
pub struct EmailClient {
    pub api_url: String,
    pub api_email: SubscriberEmail,
    pub api_key: Secret<String>,
}

impl EmailClient {
    pub fn new(api_url: String, api_email: SubscriberEmail, api_key: Secret<String>) -> Self {
        Self {
            api_url,
            api_email,
            api_key,
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        text_content: &str,
        html_content: &str,
    ) -> Result<(), reqwest::Error> {
        //Defining the email
        let email_payload = json!({
            "from": {"email" : "ramses.hdzven@gmail.com"},
            "to": [{"email": recipient.as_ref()}],
            "subject": subject,
            "text": text_content,
            "html": html_content
        });

        let client = Client::new();
        let response = client
            .post(&self.api_url)
            .header(
                "Authorization",
                format!("Bearer {}", &self.api_key.expose_secret()),
            )
            .header("Content-Type", "application/json")
            .body(email_payload.to_string())
            .send()
            .await?;

        println!("{:?}", response);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SubscriberEmail;

    #[tokio::test]
    async fn sending_email_unit_test() {
        let email = SubscriberEmail::parse("ramses@dagatech.solutions".to_string()).unwrap();
        let email_client = EmailClient::new(
            "https://sandbox.api.mailtrap.io/api/send/2755270".to_string(),
            email,
            Secret::new("06317472283fda0dc9965a525aeb539f".to_string()),
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
