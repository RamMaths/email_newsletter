use crate::helpers::TestApp;
use reqwest::Url;
use wiremock::{matchers::method, matchers::path, Mock, ResponseTemplate};

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let app = TestApp::spawn_app().await;
    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .json(&serde_json::json!({
                    "title": "Newsletter title",
                    "content": {
                        "text": "Newsletter body as plain text",
                        "html": "<p>Newsletter body as HTML</p>",
                    }
        }))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = TestApp::spawn_app().await;

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
    let app = TestApp::spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/api/send/2755270"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let news_letter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter as plain text",
            "html": "<p>Newsletter body as html</p>"
        }
    });

    println!("{}", news_letter_request_body.to_string());

    let response = app.post_newsletters(news_letter_request_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = TestApp::spawn_app().await;
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

    let _mock_guard = Mock::given(path("/api/send/2755270"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    let response = app
        .post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    assert_eq!(200, response.status().as_u16());

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    let confirmation_link = app.get_confirmation_links(&email_request);
    Url::parse(&confirmation_link).unwrap()
}
