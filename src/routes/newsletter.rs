use actix_web::http::StatusCode;
use actix_web::ResponseError;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::error_handling::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct BodyData {
    pub title: String,
    pub content: Content,
}
#[derive(serde::Deserialize)]
pub struct Content {
    pub html: String,
    pub text: String,
}

pub struct ConfirmedSubscriber {
    pub email: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&pool).await?;
    for subscriber in subscribers {
        email_client.send_email(
            SubscriberEmail::parse(subscriber.email)?,
            &body.title,
            &body.content.html,
            &body.content.text,
        );
    }
    Ok(HttpResponse::Ok().finish())
}

pub async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    let rows = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)

