use app_state::AppState;
use axum::{
    body::{boxed, Body, BoxBody, Bytes, HttpBody},
    http::{Request, Response, StatusCode, Uri},
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_sessions::{async_session, extractors::ReadableSession, SessionHandle, SessionLayer};
use models::User;
use rand::Rng;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::{services::ServeDir, trace::TraceLayer};

mod accounts;
mod app_state;
mod day;
mod events;
mod index;
mod list;
pub mod models;
mod navigation;
mod shift;
mod time_ext;
pub(crate) mod utils;

#[tokio::main]
async fn main() {
    let store = async_session::MemoryStore::new(); // Should create adapter for postgres store?
    let mut secret = [0u8; 128];
    rand::thread_rng().fill(&mut secret[..]);
    let session_layer = SessionLayer::new(store, &secret);

    let app_state = AppState::init().await;

    let auth_routes = Router::new()
        .nest("/day", day::protected_router())
        .nest("/event", events::protected_router())
        .with_state(app_state.clone())
        .layer(middleware::from_fn(auth_layer));

    let app = Router::new()
        .route("/", get(|| async { Html("<p>Hello World</p>") }))
        .route("/nav", get(navigation::navigation))
        .nest("/account", accounts::public_router())
        .route("/login", get(accounts::login_form).post(accounts::login))
        .route("/logout", get(accounts::logout))
        .with_state(app_state)
        .merge(auth_routes)
        .fallback(get_static_files)
        .layer(
            ServiceBuilder::new()
                .layer(session_layer)
                .layer(middleware::from_fn(auth_changes_layer))
                .layer(middleware::from_fn(html_wrapper))
                .layer(TraceLayer::new_for_http()),
        );

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn auth_changes_layer<B>(request: Request<B>, next: Next<B>) -> impl IntoResponse {
    let session_handle = request //use session handle so it doesn't lock it
        .extensions()
        .get::<SessionHandle>()
        .cloned()
        .expect("Session handle should exist");
    let mut res = next.run(request).await;
    if session_handle.read().await.data_changed() {
        res.headers_mut().append(
            "HX-Trigger",
            "auth-change".parse().expect("Valid header value"),
        );
    }
    res
}

async fn auth_layer<B>(
    session: ReadableSession,
    request: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    if session.is_destroyed() || session.is_expired() || session.get::<User>("user").is_none() {
        return Err((
            StatusCode::FORBIDDEN,
            Html(format!(
                r##"<span hx-get="/login?from={}" hx-trigger="load" hx-target="#content" hx-push-url="true"></span>"##,
                request.uri()
            )),
        ));
    }
    drop(session);
    Ok(next.run(request).await)
}

async fn html_wrapper<B>(request: Request<B>, next: Next<B>) -> impl IntoResponse {
    let from_htmx = request.headers().contains_key("HX-Request");
    let is_index = request.uri() == "/index.html" || request.uri() == "/index";
    let response = next.run(request).await;

    let is_html = response
        .headers()
        .get("content-type")
        .is_some_and(|ct| ct.as_bytes().starts_with(b"text/html"));
    let Html(wrapper) = navigation::index().await;

    response.map_data(move |b| {
        if !from_htmx && !is_index && is_html {
            let (header, footer) = wrapper
                .split_once("Content")
                .expect("Index.html is missing \"Content\"");

            let mut new_body = header.as_bytes().to_vec();
            new_body.extend(b);
            new_body.extend_from_slice(footer.as_bytes());
            Bytes::copy_from_slice(&new_body)
        } else {
            b
        }
    })
}

async fn get_static_files(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, Html<&'static str>)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    ServeDir::new("./public")
        .oneshot(req)
        .await
        .map_err(utils::ise)
        .map(|res| res.map(boxed))
}
