use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

pub const X_REQUEST_ID: &str = "X-Request-Id";

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    let req_id = req
        .headers()
        .get(X_REQUEST_ID)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    req.extensions_mut().insert(req_id.clone());
    let mut resp = next.run(req).await;
    resp.headers_mut()
        .insert(X_REQUEST_ID, HeaderValue::from_str(&req_id).unwrap());
    resp
}
