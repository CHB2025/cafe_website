use std::fmt::Display;

use axum::{http::StatusCode, response::Html};

pub fn ise<E: Display>(err: E) -> (StatusCode, Html<&'static str>) {
    println!("DB Error: {}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html("<span class=\"error\">Internal Server Error</span>"),
    )
}
