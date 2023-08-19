use axum::response::Html;
use axum_sessions::extractors::ReadableSession;

use crate::models::User;

pub async fn navigation(session: ReadableSession) -> Html<String> {
    let (left, right) = if !session.is_destroyed()
        && !session.is_expired()
        && session.get::<User>("user").is_some()
    {
        (
            r##"
                <a class="nav-item button" href="/event/list" hx-boost="true">Events</a>
            "##,
            r##"
                <a class="nav-item button" href="/logout" hx-boost="true">Log Out</a>
            "##,
        )
    } else {
        (
            "",
            r##"
                <a class="nav-item button" href="/login" hx-boost="true">Log In</a>
            "##,
        )
    };

    Html(format!(
        r##"
            <div id="navigation" hx-target="#content" hx-push-url="true">
                <span hx-get="/nav" hx-target="#navigation" hx-swap="outerHTML" hx-trigger="auth-change from:body"></span>
                <div class="nav-container nav-left">
                    <a class="nav-item title" href="/" hx-boost="true"><h1>Cornerstone Cafe</h1></a> 
                    {left}
                </div>
                <div class="nav-container nav-center"></div>
                <div class="nav-container nav-right">
                    {right}
                </div>
            </div>
        "##
    ))
}

pub async fn index(session: ReadableSession) -> Html<String> {
    let Html(nav) = navigation(session).await;
    Html(format!(
        r##"
            <head>
                <link rel="stylesheet" href="/styles/index.css">
                <link rel="stylesheet" href="/styles/form.css">
                <link rel="stylesheet" href="/styles/list.css">
                <script src="https://unpkg.com/htmx.org@1.9.3"></script>
                <script src="https://unpkg.com/hyperscript.org@0.9.9"></script>
            </head>
            <body>
                {nav}
                <div id="content">
                    Content
                </div>

            </body>
        "##
    ))
}
