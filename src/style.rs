use askama_axum::IntoResponse;
use axum::http::StatusCode;

pub async fn style() -> impl IntoResponse {
    const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));
    (StatusCode::OK, [("Content-Type", "text/css")], STYLESHEET)
}
