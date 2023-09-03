use askama::Template;
use axum_sessions::extractors::ReadableSession;

use crate::models::User;

#[derive(Template)]
#[template(path = "navigation.html")]
pub struct Nav {
    left: Vec<(String, String)>,
    right: Vec<(String, String)>,
}

pub async fn navigation(session: ReadableSession) -> Nav {
    let (left, right) = if session.is_destroyed()
        || session.is_expired()
        || session.get::<User>("user").is_none()
    {
        (vec![], vec![("Log In".to_string(), "/login".to_string())])
    } else {
        (
            vec![("Events".to_string(), "/event/list".to_string())],
            vec![("Log Out".to_string(), "/logout".to_string())],
        )
    };
    Nav { left, right }
}
