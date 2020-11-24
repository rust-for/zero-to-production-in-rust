// tests/health_checks

use std::net::TcpListener;

use zero2prod::startup;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use sqlx::{PgPool, PgConnection, Connection, Executor};
use uuid::Uuid;

// `actix_rt::test` is the testing equivalent of `actix_rt::main`.
// It also spares you from having to specify the `#[test] attribute.
// You can inspect what code gets generated using
// `cargo expend --test health_check` (<- name of the test file)
// #[actix_rt::test]
// async fn health_check_succeeds() {
//
//     // Arrange
//     let app = spawn_app().await;
//
//     // We brought `reqwest` in as a _development_ dependency
//     // to perform HTTP requests against our application.
//     // Either add it manually under [dev-dependencies] in Cargo.toml
//     // or run `cargo add reqwest --dev`
//     let client = reqwest::Client::new();
//
//     // Act
//     let response = client
//         .get(&format!("{}/health_check", &app.address))
//         .send()
//         .await
//         .expect("Failed to execute request.");
//
//     // Assert
//     assert!(response.status().is_success());
//     assert_eq!(Some(0), response.content_length());
// }

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    // let app_address = spawn_app();
    // let configuration = get_configuration().expect("Failed to read configuration");
    // let connection_string = configuration.database.connection_string();
    // let mut connection = PgConnection::connect(&connection_string)
    //     .await
    //     .expect("Failed to connect to Postgres.");
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=lebron%20James&email=nba_lebron_james%40gmail.com";

    // Act
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions", )
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "nba_lebron_james@gmail.com");
    assert_eq!(saved.name, "lebron James")
}

#[actix_rt::test]
async fn subscribe_return_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=lebron%20james", "missing the email"),
        ("email=nba_lebron_james%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did fail with 400 Bad Request when payload was {}.",
            error_message
        );
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    // let connection_pool = PgPool::connect(&configuration.database.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres.");
    let connection_pool = configure_database(&configuration.database).await;

    let server = startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
    // New dev dependency - let's add tokio to the party with
    // `cargo add tokio --dev vers 0.2.22`
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawn future.
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut conneciton = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect o Postgres");
    conneciton
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}