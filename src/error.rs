use std::{error::Error, fmt};

use askama::Template;
use askama_axum::IntoResponse;
use axum::http::StatusCode;

#[derive(Template, Debug)]
#[template(path = "error.html")]
struct ErrorTemplate {
    code: StatusCode,
    message: &'static str,
    kind: DisplayKind,
}

#[derive(Debug)]
pub enum DisplayKind {
    Block,
    Inline,
    Redirect(String),
}

const ISE: AppError = AppError(
    StatusCode::INTERNAL_SERVER_ERROR,
    "An unexpected error occured",
    DisplayKind::Block,
);
const NOT_FOUND: AppError = AppError(
    StatusCode::NOT_FOUND,
    "The requested resource could not be found",
    DisplayKind::Block,
);

#[derive(Debug)]
pub struct AppError(StatusCode, &'static str, DisplayKind);

impl AppError {
    pub fn inline(code: StatusCode, message: &'static str) -> AppError {
        AppError(code, message, DisplayKind::Inline)
    }

    pub fn block(code: StatusCode, message: &'static str) -> AppError {
        AppError(code, message, DisplayKind::Block)
    }

    pub fn redirect(code: StatusCode, message: &'static str, url: String) -> AppError {
        AppError(code, message, DisplayKind::Redirect(url))
    }
}

impl Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => NOT_FOUND,
            _ => ISE,
        }
    }
}

impl From<serde_urlencoded::ser::Error> for AppError {
    fn from(_: serde_urlencoded::ser::Error) -> Self {
        ISE
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(_: tokio::task::JoinError) -> Self {
        ISE
    }
}

impl From<scrypt::password_hash::Error> for AppError {
    fn from(err: scrypt::password_hash::Error) -> Self {
        match err {
            scrypt::password_hash::Error::Password => {
                Self::inline(StatusCode::BAD_REQUEST, "Invalid username or password")
            }
            _ => ISE,
        }
    }
}

impl From<askama::Error> for AppError {
    fn from(_: askama::Error) -> Self {
        ISE
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> askama_axum::Response {
        let AppError(code, message, kind) = self;
        let body = ErrorTemplate {
            code,
            message,
            kind,
        };
        let headers = [("content-type", "text/html")];
        (code, headers, body).into_response()
    }
}
