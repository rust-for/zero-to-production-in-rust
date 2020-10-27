// tests/health_check.rs

use std::net::TcpListener;

// `actix_rt::test` is the testing equivalent of `actix_rt::main`.
// It also spares you from having to specify the `#[test] attribute.
// You can inspect what code gets generated using
// `cargo expend --test health_check` (<- name of the test file)
#[actix_rt::test]
async fn health_check_succeeds() {

    // Arrange
    // spawn_app().await.expect("Failed to spawn our app");
    let address = spawn_app();

    // We brought `reqwest` in as a _development_ dependency
    // to perform HTTP requests against our application.
    // Either add it manually under [dev-dependencies] in Cargo.toml
    // or run `cargo add reqwest --dev`
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::run(listener).expect("Failed to bind address");
    // New dev dependency - let's add tokio to the party with
    // `cargo add tokio --dev vers 0.2.22`
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawn future.
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

