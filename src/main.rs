use std::{env, net::SocketAddr, time::Duration};

use app_state::AppState;
use axum::{
    body::{boxed, Body, BoxBody, Bytes, HttpBody},
    http::{Request, Response, StatusCode, Uri},
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_extra::extract::Cached;
use axum_server::tls_rustls::RustlsConfig;

use cafe_website::AppError;
use models::User;

use tower::{ServiceBuilder, ServiceExt};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::prelude::*;

mod accounts;
mod app_state;
mod config;
mod email;
mod events;
mod home;
mod index;
pub mod models;
mod navigation;
mod schedule;
mod session;
mod shift;
mod time_ext;
mod worker;

#[tokio::main]
async fn main() {
    let config_path = env::current_dir()
        .expect("Couldn't load current dir")
        .join("config.toml");
    let config = match config::Config::load(config_path).await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to parse config: {}", e);
            return;
        }
    };

    // Tracing
    // TODO: put tracing configuration in the config file
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "cafe_website=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // State
    let app_state = AppState::init(config).await;

    // Emailing!
    let email_state = app_state.clone();
    tokio::spawn(async move {
        if let Some(cfg) = &email_state.config().email {
            loop {
                if let Err(e) = email::send_all(email_state.pool(), cfg).await {
                    tracing::error!("Email error: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        } else {
            tracing::warn!("No email config, not starting");
        }
    });

    // Routes
    let auth_routes = Router::new()
        .nest("/event", events::protected_router())
        .nest("/account", accounts::protected_router())
        .nest("/shift", shift::protected_router())
        .nest("/worker", worker::protected_router())
        .nest("/email", email::protected_router())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_layer,
        ));

    let public_routes = Router::new()
        .route("/", get(home::view))
        .route("/nav", get(navigation::navigation))
        .route("/login", get(accounts::login_form).post(accounts::login))
        .route("/logout", get(accounts::logout))
        .nest("/event", events::public_router())
        .nest("/account", accounts::public_router())
        .nest("/shift", shift::public_router())
        .nest("/worker", worker::public_router())
        .nest("/email", email::public_router());

    // App
    let app = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .with_state(app_state.clone())
        .fallback(get_static_files)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(html_wrapper))
                .layer(TraceLayer::new_for_http()),
        );

    let port = app_state.config().website.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::debug!("listening on {}", addr);

    match &app_state.config().ssl {
        Some(cfg) => {
            let rustls_config = RustlsConfig::from_pem_file(cfg.cert.clone(), cfg.key.clone())
                .await
                .expect("Invalid Certs");

            axum_server::bind_rustls(addr, rustls_config)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        None => axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .unwrap(),
    };
}

async fn auth_layer<B>(
    user_session: Cached<Option<User>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response<BoxBody>, AppError> {
    if user_session.is_none() {
        return Err(AppError::redirect(
            StatusCode::UNAUTHORIZED,
            "Restricted",
            format!(
                "/login?from={}",
                request.uri().path_and_query().map_or("", |p| p.as_str())
            ),
        ));
    }
    let mut res = next.run(request).await;
    res.headers_mut().insert(
        "Cache-Control",
        "no-store".parse().expect("No-store is valid header value"),
    );
    Ok(res)
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

async fn get_static_files(uri: Uri) -> Result<Response<BoxBody>, AppError> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    ServeDir::new("./public")
        .oneshot(req)
        .await
        .map_err(|_| cafe_website::error::ISE)
        .map(|mut res| {
            res.headers_mut().append(
                "Cache-Control",
                "max-age=31536000, immutable"
                    .parse()
                    .expect("Cache header is valid"),
            );
            res.map(boxed)
        })
}
