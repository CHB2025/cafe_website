use std::{error::Error, fmt};

use askama::Template;
use axum::{
    http::{uri::Parts, Response, StatusCode},
    response::IntoResponse,
};

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
}

const ISE: AppError = AppError(
    StatusCode::INTERNAL_SERVER_ERROR,
    "An unexpected error occured",
    DisplayKind::Block,
);
const NOT_FOUND: AppError = AppError(
    StatusCode::INTERNAL_SERVER_ERROR,
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
