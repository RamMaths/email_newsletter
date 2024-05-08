use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String
}

struct UserData {
    subscriber_id: Uuid,
    status: String
}

#[tracing::instrument(
    name = "Confirm a pending subscriber"
    skip(parameters, pool)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>
) -> HttpResponse {

    let user = match get_subscriber_from_token(&pool, &parameters.subscription_token).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };

    match user {
        // Non existing token
        None => return HttpResponse::Unauthorized().finish(),
        Some(user) => {
            if user.status == "confirmed" {
                return HttpResponse::Conflict().finish();
            }

            if confirm_subscriber(&pool, user.subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }

            HttpResponse::Ok().finish()
        }
    }
}

#[tracing::instrument(
    name = "Get subscriber from token",
    skip(subscription_token, pool)
)]
pub async fn get_subscriber_from_token(
    pool: &PgPool,
    subscription_token: &str
) -> Result<Option<UserData>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id, s.status \
        FROM subscription_tokens AS st \
        JOIN subscriptions AS s ON s.id = st.subscriber_id \
        WHERE st.subscription_token LIKE $1",
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map(|e| {
        tracing::error!("Failed to execute the query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| UserData { subscriber_id: r.subscriber_id, status: r.status }))
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(subscriber_id, pool)
)]
pub async fn confirm_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid
) -> 
Result<(), sqlx::Error> {

    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id=$1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
