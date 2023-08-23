use askama::Template;
use axum::response::Html;
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

pub async fn index() -> Html<String> {
    Html(
        r##"
            <head>
                <link rel="stylesheet" href="/styles/index.css">
                <link rel="stylesheet" href="/styles/form.css">
                <link rel="stylesheet" href="/styles/list.css">
                <script src="https://unpkg.com/htmx.org@1.9.3"></script>
                <script src="https://unpkg.com/hyperscript.org@0.9.9"></script>
            </head>
            <body>
                <div id="navigation" hx-get="/nav" hx-swap="outerHTML" hx-trigger="load"></div>
                <div id="content">
                    Content
                </div>

            </body>
        "##
        .to_string(),
    )
}
