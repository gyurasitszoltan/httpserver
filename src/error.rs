use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

pub(crate) type R<T> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub(crate) enum E {
    #[error("{0}")]
    Bad(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Conflict(String),
    #[error("rate limit")]
    Rate,
    #[error("{0}")]
    Internal(String),
}

impl From<sqlx::Error> for E {
    fn from(error_value: sqlx::Error) -> Self {
        error!(%error_value, "sqlx");
        Self::Internal("Adatbázis hiba".into())
    }
}

impl IntoResponse for E {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Bad(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Rate => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let message = match &self {
            Self::Bad(value) | Self::Conflict(value) => value.clone(),
            Self::Unauthorized => "Bejelentkezés szükséges.".into(),
            Self::Forbidden => "Nincs jogosultságod ehhez a művelethez.".into(),
            Self::NotFound => "A kért erőforrás nem található.".into(),
            Self::Rate => "Túl sok kérés. Próbáld később.".into(),
            Self::Internal(value) => {
                error!(%value, "api error");
                "Váratlan szerverhiba történt.".into()
            }
        };
        (
            status,
            Json(serde_json::json!({"error":{"code":status.as_str(),"message":message}})),
        )
            .into_response()
    }
}
