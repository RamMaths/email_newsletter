# EMAIL NEWSLETTER

Simple though robust crud backend service written in rust. It's going to integrate must of the functionality a *Cloud Native* application should have. The application will only listen to HTTP Requests and it's going to respond with an HTTP Response, that's it.
 
## Web framework

[Actix-web](https://actix.rs) is one of Rust’s oldest frameworks. It has seen extensive production usage, and it has built a large community and plugin ecosystem; it runs on [tokio](https://tokio.rs), an asynchronous rust runtime.

## Database
This project uses [PostgreSQL](https://www.postgresql.org) since it is widely supported across all cloud providers, opensource, exhaustive documentation, easy to run locally and in CI via Docker, well-supported within the Rust ecosystem. The [sqlx](https://crates.io/crates/sqlx) crate will help us communicating with the database, sqlx, uses procedural macros to connect to a database at compile-time and check if the provided query is indeed sound. The database runs under a docker container.

### Database migrations
To add new tables we need to change its schema, this is commonly referred to as a *database migration*.

This is how you would create a migration

```bash
# Assuming you used the default parameters to launch Postgres in Docker!
export DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/newsletter
sqlx migrate add create_subscriptions_table
```

then you write your queries inside the generated file.

```sql
-- migrations/{timestamp}_create_subscriptions_table.sql
-- Create Subscriptions Table
CREATE TABLE subscriptions(
   id uuid NOT NULL,
   PRIMARY KEY (id),
   email TEXT NOT NULL UNIQUE,
   name TEXT NOT NULL,
   subscribed_at timestamptz NOT NULL
);
```
## Project structure

### Health check
The application has an endpoint `/health_check` which only ensures availability of the service.

```rs
use actix_web::HttpResponse;

// health check
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
```

In order to migrate the script you can do `sqlx migrate run` in the terminal but for integrated test we use the `sqlx::migrate!("path/to/migrations")` macro.

## Test driven development
Manual testing is time consuming as an application gets bigger, it gets more expensive to manually check all our assumptions. Therefore I'll try to put a strong emphasis on test-driven development and continuous integration.

All *integrations tests* are going to be inside the `/tests` folder, while *unit tests* will be placed in each file a unit test is required.

### Test isolation
When we run our tests that interacts with the database we don't want to mess up the actual stored data, that's why we need to isolate the tests. We create a new logical database and then we run migrations on it on every test that requires a database.

```rs
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
    .await
    .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, &config.database_name).as_str())
        .await
        .expect("Failed to create database");

    //Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
```

## Telemetry

Telemetry data is all the information about our running application that is collected automatically that can be later inspected to answer questions about the state of the system. We're relying on the following dependencies:

```rs
[dependencies]
tracing = { version = "0.1", features = ["log"] }
tracing-subscriber = { version = "0.3", features = ["registry", "env-filter"] }
tracing-bunyan-formatter = "0.3"
tracing-log = "0.1"
tracing-actix-web = "0.7.10"
once_cell = "1.19.0"
```

We basically set a subscriber (check out `src/telemetry.rs`), by setting a subscriber, then we wrap every function we want to log in a span.

```rs
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection),
    fields(
        subscriber_email = &form.email,
        subscriber_name = &form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse {
    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        Err(_) => return HttpResponse::BadRequest().finish()
    };

    let new_subscriber = NewSubscriber {
        email: form.0.email,
        name
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
```

## Deployment

### Building the docker image and runnning it locally

```bash
docker build --tag email_newsletter --file Dockerfile .
docker run --rm -p 8000:8000 email_newsletter | bunyan
```
### Deploying to digital Ocean

We need to add some configurations while pushing the application to the cloud provider such as the github repository that is going to take in order to build the image that is going to run the instance or the database configuration. All these setting go inside the `spec.yaml` file.

```bash
doctl apps create --spec spec.yaml
doctl apps list
```

If you hit an endpoint that uses the database you will get an error at this stage if you haven't migrated the database.

```bash
DATABASE_URL=YOUR-DIGITAL-OCEAN-DB-CONNECTION-STRING sqlx migrate run
```

## Serde for serializing
Serde is a framework for serializing and deserializing Rust data structures efficiently and generically. This will help us serializing the input of an incoming request, reading configuration files and more.

### Example to deserialize a configuration file

```rs
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool
}
```

(configutation file with .yaml extension)
```yaml
database:
  host: "127.0.0.1"
  port: 5432
  username: "username"
  password: "password"
  database_name: "database_name"
```
