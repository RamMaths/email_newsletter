use crate::helpers::TestApp;

pub async fn create_unconfirmed_subscriber(app: &TestApp) {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
}
