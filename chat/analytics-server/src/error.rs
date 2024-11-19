use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Json;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorOutput {
    pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("clickhouse error: {0}")]
    ClickhouseError(#[from] clickhouse::error::Error),

    #[error("any error: {0}")]
    AnyError(#[from] anyhow::Error),

    #[error("miss event context")]
    MissEventContext,

    #[error("miss event data")]
    MissEventData,
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::ClickhouseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AnyError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::MissEventContext => StatusCode::BAD_REQUEST,
            AppError::MissEventData => StatusCode::BAD_REQUEST,
        };

        (status, Json(json!(ErrorOutput::new(self.to_string())))).into_response()
    }
}
