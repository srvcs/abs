use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router as AxumRouter};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use srvcs_abs::{api::Deps, health, router, telemetry};
use tower::ServiceExt;

/// Mock dependency answering `POST /` with a fixed status + body.
async fn spawn_mock(status: StatusCode, body: Value) -> String {
    let app = AxumRouter::new().route(
        "/",
        post(move || {
            let body = body.clone();
            async move { (status, Json(body)) }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn app(isnegative_url: &str, negate_url: &str) -> axum::Router {
    router(
        telemetry::metrics_handle_for_tests(),
        Deps {
            isnegative_url: isnegative_url.to_string(),
            negate_url: negate_url.to_string(),
        },
    )
}

async fn eval(isnegative_url: &str, negate_url: &str, value: Value) -> (StatusCode, Value) {
    let res = app(isnegative_url, negate_url)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "value": value }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

const DEAD_URL: &str = "http://127.0.0.1:1";

async fn status_of(uri: &str) -> StatusCode {
    app(DEAD_URL, DEAD_URL)
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn healthz_ok() {
    assert_eq!(status_of("/healthz").await, StatusCode::OK);
}

#[tokio::test]
async fn readyz_reflects_state() {
    health::set_ready(true);
    assert_eq!(status_of("/readyz").await, StatusCode::OK);
}

#[tokio::test]
async fn openapi_ok() {
    assert_eq!(status_of("/openapi.json").await, StatusCode::OK);
}

#[tokio::test]
async fn returns_value_unchanged_when_not_negative() {
    // isnegative says false; negate must never be consulted.
    let isnegative = spawn_mock(StatusCode::OK, json!({ "result": false })).await;
    let (status, body) = eval(&isnegative, DEAD_URL, json!(6)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["value"], json!(6));
    assert_eq!(body["result"], json!(6));
}

#[tokio::test]
async fn negates_when_negative() {
    let isnegative = spawn_mock(StatusCode::OK, json!({ "result": true })).await;
    let negate = spawn_mock(StatusCode::OK, json!({ "result": 6 })).await;
    let (status, body) = eval(&isnegative, &negate, json!(-6)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["value"], json!(-6));
    assert_eq!(body["result"], json!(6));
}

#[tokio::test]
async fn forwards_invalid_input_from_isnegative() {
    let isnegative = spawn_mock(
        StatusCode::UNPROCESSABLE_ENTITY,
        json!({ "error": "value is not an integer" }),
    )
    .await;
    let (status, _) = eval(&isnegative, DEAD_URL, json!(4.5)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn forwards_invalid_input_from_negate() {
    let isnegative = spawn_mock(StatusCode::OK, json!({ "result": true })).await;
    let negate = spawn_mock(
        StatusCode::UNPROCESSABLE_ENTITY,
        json!({ "error": "value is not an integer" }),
    )
    .await;
    let (status, _) = eval(&isnegative, &negate, json!(-4)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn degrades_when_isnegative_is_unreachable() {
    let (status, body) = eval(DEAD_URL, DEAD_URL, json!(-6)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["dependency"], "srvcs-isnegative");
}

#[tokio::test]
async fn degrades_when_negate_is_unreachable() {
    let isnegative = spawn_mock(StatusCode::OK, json!({ "result": true })).await;
    let (status, body) = eval(&isnegative, DEAD_URL, json!(-6)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["dependency"], "srvcs-negate");
}
