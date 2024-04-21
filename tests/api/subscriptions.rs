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

