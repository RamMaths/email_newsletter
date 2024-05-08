use crate::helpers::*;
use reqwest::Url;
use email_newsletter::email_client::TestResponse;

#[tokio::test]
async fn subscribing_through_smtp() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Creating a new subscriber for the first time

    let response = app
        .post_subscriptions(body.into())
        .await;

    assert_eq!(200, response.status().as_u16());

    let response = response
        .json::<TestResponse>()
        .await
        .expect("Coludn't parse the json response");

    let url = Url::parse(&app.address)
        .expect("Couldn't parse the link")
        .join(&response.text)
        .expect("Couldn't parse the link");

    println!("{}", url);

    // confirming the email
    reqwest::get(url)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}

#[tokio::test]
async fn inserting_a_subscriber_twice() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Creating a new subscriber for the first time

    let response = app
        .post_subscriptions(body.into())
        .await;

    assert_eq!(200, response.status().as_u16());

    let response = response
        .json::<TestResponse>()
        .await
        .expect("Coludn't parse the json response");

    let url = Url::parse(&app.address)
        .expect("Couldn't parse the link")
        .join(&response.text)
        .expect("Couldn't parse the link");

    println!("{}", url);

    // Trying to save the subscriber twice

    let response = app
        .post_subscriptions(body.into())
        .await;

    assert_eq!(200, response.status().as_u16());

    let response = response
        .json::<TestResponse>()
        .await
        .expect("Coludn't parse the json response");

    let url = Url::parse(&app.address)
        .expect("Couldn't parse the link")
        .join(&response.text)
        .expect("Couldn't parse the link");

    println!("{}", url);

    // confirming the email

    reqwest::get(url)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn using_a_confirmation_token_twice_returns_409() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Creating a new subscriber for the first time

    let response = app
        .post_subscriptions(body.into())
        .await;

    assert_eq!(200, response.status().as_u16());

    let response = response
        .json::<TestResponse>()
        .await
        .expect("Coludn't parse the json response");

    let url = Url::parse(&app.address)
        .expect("Couldn't parse the link")
        .join(&response.text)
        .expect("Couldn't parse the link");
    
    // Using a confirmation link twice

    let _ = reqwest::get(url.clone())
        .await
        .unwrap();
    
    let response: u16 = reqwest::get(url)
        .await
        .unwrap()
        .status().
        as_u16();

    assert_eq!(response, 409);
}
