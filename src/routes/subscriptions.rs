use actix_web::{ 
    web,
    HttpResponse
};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use crate::{
    domain::NewSubscriber,
    email_client::EmailClient
};

// subscribe
#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection),
    fields(
        subscriber_email = &form.email,
        subscriber_name = &form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>, email_client: web::Data<EmailClient>) -> HttpResponse {
    
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish()
    };


    if insert_subscriber(&new_subscriber, &connection).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client, new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, connection),
    fields(
        error_message = "",
        event_type = "[SAVING NEW SUBSCRIBER DETAILS IN THE DATABASE - EVENT]"
    ),
    err
)]
pub async fn insert_subscriber(new_subscriber: &NewSubscriber, connection: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute the query: {}", e);
        e
    })?;
    
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber
) -> Result<(), Box<dyn std::error::Error>> {

    let confirmation_link = "https://there-is-no-such-domain.com/subscriptions/confirm";

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
    .await?;

    Ok(())
}

