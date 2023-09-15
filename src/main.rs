use std::{net::SocketAddr, path::PathBuf};

use app_state::AppState;
use axum::{
    body::{boxed, Body, BoxBody, Bytes, HttpBody},
    http::{Request, Response, StatusCode, Uri},
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use axum_sessions::{async_session, extractors::ReadableSession, SessionHandle, SessionLayer};
use error::AppError;
use models::User;
use rand::Rng;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::prelude::*;

mod accounts;
mod app_state;
mod error;
mod events;
mod home;
mod index;
pub mod models;
mod navigation;
mod schedule;
mod shift;
mod time_ext;
pub(crate) mod utils;

#[tokio::main]
async fn main() {
    // Sessions
    let store = async_session::MemoryStore::new(); // Should create adapter for postgres store?
    let mut secret = [0u8; 128];
    rand::thread_rng().fill(&mut secret[..]);
    let session_layer = SessionLayer::new(store, &secret);

    // Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "cafe_website=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // State
    let app_state = AppState::init().await;

    // Routes
    let auth_routes = Router::new()
        .nest("/schedule", schedule::protected_router())
        .nest("/event", events::protected_router())
        .nest("/account", accounts::protected_router())
        .nest("/shift", shift::protected_router())
        .layer(middleware::from_fn(auth_layer));

    let public_routes = Router::new()
        .route("/", get(home::view))
        .route("/nav", get(navigation::navigation))
        .route("/login", get(accounts::login_form).post(accounts::login))
        .route("/logout", get(accounts::logout))
        .nest("/event", events::public_router())
        .nest("/schedule", schedule::public_router())
        .nest("/account", accounts::public_router())
        .nest("/shift", shift::public_router());

    // App
    let app = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .with_state(app_state)
        .fallback(get_static_files)
        .layer(
            ServiceBuilder::new()
                .layer(session_layer)
                .layer(middleware::from_fn(auth_changes_layer))
                .layer(middleware::from_fn(html_wrapper))
                .layer(TraceLayer::new_for_http()),
        );

    // TLS config
    // Currently errors out if certs aren't there, but could be setup to run https only if they are
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certs")
            .join("cert.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certs")
            .join("key.pem"),
    )
    .await
    .unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::debug!("listening on {}", addr);
    axum_server::bind_rustls(addr, config)
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
) -> Result<Response<BoxBody>, AppError> {
    if session.is_destroyed() || session.is_expired() || session.get::<User>("user").is_none() {
        return Err(AppError::redirect(
            StatusCode::UNAUTHORIZED,
            "Restricted",
            format!("/login?from={}", request.uri()),
        ));
    }
    drop(session);
    Ok(next.run(request).await)
}

async fn html_wrapper<B>(request: Request<B>, next: Next<B>) -> impl IntoResponse {
    let from_htmx = request.headers().contains_key("HX-Request");
    let response = next.run(request).await;

    let Html(wrapper) = index::index().await;

    let (mut parts, mut body) = response.into_parts();
    parts.headers.remove("content-length");

    let is_html = parts
        .headers
        .get("content-type")
        .is_some_and(|ct| ct.as_bytes().starts_with(b"text/html"));

    body = body
        .map_data(move |b| {
            if !from_htmx && is_html {
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
        .boxed_unsync();

    Response::from_parts(parts, body)
}

async fn get_static_files(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, Html<&'static str>)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    ServeDir::new("./public")
        .oneshot(req)
        .await
        .map_err(utils::ise)
        .map(|res| res.map(boxed))
}
