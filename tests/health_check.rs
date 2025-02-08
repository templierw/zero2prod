use std::net::TcpListener;

use sqlx::{Connection, Executor, PgConnection, PgPool};

use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DataBaseSettings},
    startup::run,
};

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let addr = spawn_app().await.address;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &addr))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_with_valid_form_200() {
    // arrange
    let app_rs = spawn_app().await;

    let client = reqwest::Client::new();

    // act
    let body = "name=the%20Wall&email=theWall%40tqc.ca";
    let response = client
        .post(&format!("{}/subscribe", &app_rs.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app_rs.db_pool)
        .await
        .expect("failed to fetch saved metadata");

    // assert
    assert_eq!(200, response.status().as_u16());
    assert_eq!(saved.email, "theWall@tqc.ca");
    assert_eq!(saved.name, "the Wall");
}

#[tokio::test]
async fn subscribe_invalid_form_400() {
    // arrange
    let app_rs = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=the%20Wall", "missing the email"),
        ("email=theWall%40tqc.ca", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalib_body, error_message) in test_cases {
        //act
        let response = client
            .post(&format!("{}/subscribe", &app_rs.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalib_body)
            .send()
            .await
            .expect("failed to execute request");

        // assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let lst = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    let port = lst.local_addr().unwrap().port();
    let addr = format!("http://127.0.0.1:{}", port);

    let mut config = get_configuration().expect("Failed to read configuration.");
    config.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_db(&config.database).await;

    let server = run(lst, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address: addr,
        db_pool: connection_pool,
    }
}

pub async fn configure_db(config: &DataBaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_no_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
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
