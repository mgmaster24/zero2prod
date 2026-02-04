use ::zero2prod::startup::run;
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::configuration::get_config;

struct ReqwestClient {
    client: reqwest::Client,
    end_point: String,
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
    let address = spawn_app();
    let rc = ReqwestClient::new(&address, "health_check");
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
    let addr = spawn_app();
    let config = get_config().expect("Failed to read configuration");
    let connection_string = config.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres");
    let rc = ReqwestClient::new(&addr, "subscriptions");
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
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "mgmastersonjr@gmail.com");
    assert_eq!(saved.name, "michael masterson");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let addr = spawn_app();
    let rc = ReqwestClient::new(&addr, "subscriptions");
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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address");
    tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
