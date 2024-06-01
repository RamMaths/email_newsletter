use crate::error_handling::StoreTokenError;
use crate::error_handling::SubscribeError;
use crate::templates;
use crate::{
    configuration::Environment, domain::NewSubscriber, email_client::EmailClient,
    email_client::TestResponse, startup::ApplicationBaseUrl,
};
use actix_web::{web, HttpResponse};
use anyhow::Context;
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

// subscribe
#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
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
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber: NewSubscriber = form
        .0
        .try_into()
        .map_err(|err| SubscribeError::ValidationError(err))?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let mut already_exists = false;
    let subscriber_id = match insert_subscriber(&new_subscriber, &mut transaction).await {
        Err(err) => {
            if let sqlx::error::ErrorKind::UniqueViolation = err
                .as_database_error()
                .expect("Failed to cast into database error")
                .kind()
            {
                let id = get_subscriber_id(new_subscriber.name.as_ref(), &pool)
                    .await
                    .context("Failed to get the subscriber from the database")?;
                already_exists = true;

                transaction = pool
                    .begin()
                    .await
                    .context("Failed to start a transaction")?;

                id
            } else {
                return Err(SubscribeError::UnexpectedError(err.into()));
            }
        }

        Ok(id) => id,
    };

    let subscription_token = generate_subscription_token();

    if already_exists {
        update_token(&mut transaction, subscriber_id, &subscription_token)
            .await
            .context("Failed to update the confirmation token in the database")?;
    } else {
        store_token(&mut transaction, subscriber_id, &subscription_token)
            .await
            .context("Failed to store the confirmation token in the database")?;
    }

    // send_confirmation_email(
    //     &email_client,
    //     new_subscriber,
    //     &base_url.0,
    //     &subscription_token,
    // )
    // .await?;

    transaction
        .commit()
        .await
        .context("Failed to commit the SQL transaction to store a new subscriber")?;

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    match environment {
        Environment::Testing => {
            let content = format!(
                "/subscriptions/confirm?subscription_token={}",
                &subscription_token
            );

            let request_body = TestResponse {
                from: email_client.from.as_ref().to_string(),
                to: new_subscriber.email.as_ref().to_string(),
                subject: "New subscriber".into(),
                text: content.into(),
            };

            HttpResponse::Ok().json(request_body)
        }
        _ => {
            send_confirmation_email(
                &email_client,
                new_subscriber,
                &base_url.0,
                &subscription_token,
            )
            .await?;

            HttpResponse::Ok().finish()
        }
    };

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    let query = sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    );

    transaction.execute(query).await.map_err(|err| {
        tracing::error!("Failed to execute the query: {:?}", err);
        StoreTokenError(err)
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Update subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn update_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"UPDATE subscription_tokens SET subscription_token=$1 WHERE subscriber_id=$2"#,
        subscription_token,
        subscriber_id
    );

    transaction.execute(query).await.map_err(|err| {
        tracing::error!("{}", err);
        err
    })?;

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
    transaction: &mut Transaction<'_, Postgres>,
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
    token: &str,
) -> Result<(), SubscribeError> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, token
    );
    let html = templates::generate_html_template(&new_subscriber, &confirmation_link)
        .context("Failed to generate the html template for the confirmation email")?;

    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &html,
            &format!(
                "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
                confirmation_link
            ),
        )
        .await
        .context("Failed to send confirmation email to the user")?;

    Ok(())
}

#[tracing::instrument(name = "Getting a user already exists", skip(name, pool))]
pub async fn get_subscriber_id(name: &str, pool: &PgPool) -> Result<uuid::Uuid, sqlx::Error> {
    let result = sqlx::query!("SELECT id FROM subscriptions WHERE name=$1", name)
        .fetch_one(pool)
        .await?;

    Ok(result.id)
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
