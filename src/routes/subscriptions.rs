use actix_web::{ 
    web,
    HttpResponse
};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use crate::domain::NewSubscriber;

// suscribe
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
pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse {
    
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish()
    };


    match insert_subscriber(&new_subscriber, &connection).await {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!("Failed to execute the query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
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
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
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
