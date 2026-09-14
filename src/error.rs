use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found")]
    NotFound,
    #[error("internal server error")]
    Internal,
    #[error("validation failed: {0}")]
    ValidationError(String),
}

#[derive(Serialize)]
#[allow(dead_code)]
struct ErrorResp {
    code: u16,
    msg: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, 400, m.clone()),
            AppError::ValidationError(m) => (StatusCode::BAD_REQUEST, 400, m.clone()),
            AppError::NotFound => (StatusCode::NOT_FOUND, 404, "resource not found".into()),
            AppError::Internal => {
                tracing::error!(error=?self, "unhandled internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    500,
                    "internal error, please contact admin".into(),
                )
            }
        };
        (status, Json(ErrorResp { code, msg })).into_response()
    }
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, AppError>;
