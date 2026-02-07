use ::zero2prod::startup::run;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_config, DatabaseSettings};

struct ReqwestClient {
    client: reqwest::Client,
    end_point: String,
}

struct TestApp {
    address: String,
    db_pool: PgPool,
}

impl ReqwestClient {
    pub fn new(addr: &str, ep: &str) -> Self {
        ReqwestClient {
            client: reqwest::Client::new(),
            end_point: format!("{}/{}", addr, ep),
        }
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let rc = ReqwestClient::new(&app.address, "health_check");
    println!("Request endpoint: {}", rc.end_point);
    let response = rc
        .client
        .get(rc.end_point)
        .send()
        .await
        .expect("Failed to execute health check request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let rc = ReqwestClient::new(&app.address, "subscriptions");
    let body = "name=michael%20masterson&email=mgmastersonjr%40gmail.com";
    let response = rc
        .client
        .post(rc.end_point)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscription")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "mgmastersonjr@gmail.com");
    assert_eq!(saved.name, "michael masterson");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let rc = ReqwestClient::new(&app.address, "subscriptions");
    let test_cases = vec![
        ("name=michael%20masterson", "missing email"),
        ("email=mgmaster@gmail.com", "missing name"),
        ("", "missing name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = rc
            .client
            .post(&rc.end_point)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 bad Request when the payload was {}.",
            error_message
        );
    }
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut config = get_config().expect("Failed to read configuration");
    config.database.database_name = Uuid::new_v4().to_string();
    let pool = configure_db(&config.database).await;
    let server = run(listener, pool.clone()).expect("Failed to bind address");
    tokio::spawn(server);
    TestApp {
        address,
        db_pool: pool,
    }
}

async fn configure_db(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");
    pool
}
