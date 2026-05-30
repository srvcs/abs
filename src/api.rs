use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::{OpenApi, ToSchema};

use crate::client::{self, DepError};

pub const SERVICE: &str = "srvcs-abs";
pub const CONCERN: &str = "absolute value";
pub const DEPENDS_ON: &[&str] = &["srvcs-isnegative", "srvcs-negate"];

/// Dependency endpoints, injected as router state so tests can point them at
/// mock services.
#[derive(Clone)]
pub struct Deps {
    pub isnegative_url: String,
    pub negate_url: String,
}

#[derive(Serialize, ToSchema)]
pub struct Info {
    pub service: &'static str,
    pub concern: &'static str,
    pub depends_on: Vec<&'static str>,
}

/// `GET /` — service identity (srvcs service standard).
#[utoipa::path(get, path = "/", responses((status = 200, body = Info)))]
pub async fn index() -> Json<Info> {
    Json(Info {
        service: SERVICE,
        concern: CONCERN,
        depends_on: DEPENDS_ON.to_vec(),
    })
}

#[derive(Deserialize, ToSchema)]
pub struct EvalRequest {
    #[schema(value_type = Object)]
    pub value: Value,
}

#[derive(Serialize, ToSchema)]
pub struct EvalResponse {
    #[schema(value_type = Object)]
    pub value: Value,
    #[schema(value_type = Object)]
    pub result: Value,
}

fn ok(value: Value, result: Value) -> Response {
    (
        StatusCode::OK,
        Json(json!({ "value": value, "result": result })),
    )
        .into_response()
}

fn degraded(dependency: &str) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": "dependency unavailable", "dependency": dependency })),
    )
        .into_response()
}

/// Forward a dependency's response verbatim (used to propagate `422` for invalid
/// input from a leaf dependency).
fn forward(status: u16, body: Value) -> Response {
    let code = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
    (code, Json(body)).into_response()
}

/// Ask a dependency for its result, mapping its failures to the response this
/// service should return. The caller extracts `result` from the returned body.
async fn ask(url: &str, body: &Value, dependency: &str) -> Result<Value, Response> {
    match client::call(url, body).await {
        Err(DepError::Unreachable) => Err(degraded(dependency)),
        Ok((200, body)) => Ok(body.get("result").cloned().unwrap_or(Value::Null)),
        // Invalid input — forwarded from the leaf dependency.
        Ok((422, body)) => Err(forward(422, body)),
        Ok(_) => Err(degraded(dependency)),
    }
}

/// `POST /` — the absolute value of `value`.
///
/// This service does no arithmetic of its own. It asks `srvcs-isnegative`
/// whether `value` is negative; if so it asks `srvcs-negate` to flip the sign
/// and returns that. Otherwise the value is already its own absolute value.
#[utoipa::path(
    post,
    path = "/",
    request_body = EvalRequest,
    responses(
        (status = 200, body = EvalResponse),
        (status = 422, description = "value is not a valid integer (forwarded from a dependency)"),
        (status = 503, description = "a dependency is unavailable")
    )
)]
pub async fn evaluate(State(deps): State<Deps>, Json(req): Json<EvalRequest>) -> Response {
    let neg_result = match ask(
        &deps.isnegative_url,
        &json!({ "value": req.value }),
        "srvcs-isnegative",
    )
    .await
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let neg = neg_result.as_bool().unwrap_or(false);

    if !neg {
        return ok(req.value.clone(), req.value);
    }

    let result = match ask(
        &deps.negate_url,
        &json!({ "value": req.value }),
        "srvcs-negate",
    )
    .await
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    ok(req.value, result)
}

#[derive(OpenApi)]
#[openapi(
    paths(index, evaluate),
    components(schemas(Info, EvalRequest, EvalResponse))
)]
pub struct ApiDoc;

/// Serve OpenAPI document
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_documents_routes() {
        let doc = ApiDoc::openapi();
        let root = doc.paths.paths.get("/").expect("path / present");
        assert!(root.get.is_some());
        assert!(root.post.is_some());
    }

    #[tokio::test]
    async fn index_reports_both_dependencies() {
        let Json(info) = index().await;
        assert_eq!(info.service, "srvcs-abs");
        assert_eq!(info.depends_on, vec!["srvcs-isnegative", "srvcs-negate"]);
    }
}
