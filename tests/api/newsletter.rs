use crate::helpers::{spawn_app, TestApp};
use email_newsletter::email_client::TestResponse;
use reqwest::Url;

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>",
            } }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    let news_letter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter as plain text",
            "html": "<p>Newsletter body as html</p>"
        }
    });

    let response = app.post_newsletters(news_letter_request_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    let news_letter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter as plain text",
            "html": "<p>Newsletter body as html</p>"
        }
    });

    let response = app.post_newsletters(news_letter_request_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

pub async fn create_confirmed_subscriber(app: &TestApp) {
    let link = create_unconfirmed_subscriber(app).await;
    reqwest::get(link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

pub async fn create_unconfirmed_subscriber(app: &TestApp) -> Url {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = app.post_subscriptions(body.into()).await;
    println!("{:#?}", response);
    assert_eq!(200, response.status().as_u16());

    let response = response
        .json::<TestResponse>()
        .await
        .expect("Coludn't parse the json response");

    println!("{}", &response.text);

    Url::parse(&app.address)
        .expect("Couldn't parse the link")
        .join(&response.text)
        .expect("Couldn't parse the link")
}
