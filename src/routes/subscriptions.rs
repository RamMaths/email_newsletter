use actix_web::{ 
    web,
    HttpResponse
};
use sqlx::{PgPool ,Transaction, Postgres, Executor};
use chrono::Utc;
use uuid::Uuid;
use crate::{
    domain::NewSubscriber,
    email_client::EmailClient,
    startup::ApplicationBaseUrl
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

// subscribe
#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = &form.email,
        subscriber_name = &form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>
) -> HttpResponse {
    
    let new_subscriber: NewSubscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish()
    };

    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };

    let subscriber_id = match insert_subscriber(&new_subscriber, &mut transaction).await {
        Err(_) => return HttpResponse::InternalServerError().finish(),
        Ok(id) => id
    };

    let subscription_token = generate_subscription_token();

    if store_token(&mut transaction, subscriber_id, &subscription_token).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token
    )
    .await
    .is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name="Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token (
    transaction: &mut Transaction<'_,  Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    );

    transaction.execute(query).await?;

    Ok(())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction),
    fields(
        error_message = "",
        event_type = "[SAVING NEW SUBSCRIBER DETAILS IN THE DATABASE - EVENT]"
    ),
    err
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();

    let query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        &subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    );

    transaction.execute(query).await?;

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber),
    err
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    token: &str
) -> Result<(), Box<dyn std::error::Error>> {

    let confirmation_link = format!("{}/subscriptions/confirm?subscription_token={}", base_url, token);

    email_client.send_email(
        new_subscriber.email,
        "Welcome!",
        &format!(
            "Welcome to out newsletter!<br />\
            Click <a href=\"{}\">here</a> to confirm your subscription.",
            confirmation_link
        ),
        &format!(
            "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
            confirmation_link
        )
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute the query: {}", e);
        e
    })?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
