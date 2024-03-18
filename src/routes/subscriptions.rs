use actix_web::{ 
    web,
    HttpResponse
};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;

// suscribe
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String
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
    match insert_subscriber(&form, &connection).await {
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
    skip(form, connection)
)]
pub async fn insert_subscriber(form: &FormData, connection: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
