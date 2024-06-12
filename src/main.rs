use std::{env, net::SocketAddr, time::Duration};

use axum::{
    body::{boxed, Body, BoxBody, Bytes, HttpBody},
    extract::MatchedPath,
    http::{Request, StatusCode, Uri},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use session::Session;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::{services::ServeDir, trace::TraceLayer};

use cafe_website::AppError;
pub use config::config;
use tracing::{debug, info_span};

mod accounts;
mod config;
mod email;
mod events;
mod home;
mod index;
pub mod models;
mod navigation;
mod otel;
mod remind;
mod schedule;
mod session;
mod shift;
mod style;
mod time_ext;
mod worker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = env::current_dir()
        .expect("Couldn't load current dir")
        .join("config.toml");
    config::Config::init(config_path).await?;
    debug!("Config Loaded!");

    otel::init_tracing_subscriber();

    // Emailing!
    tokio::spawn(async move {
        let config = config::config();
        if config.mailer().is_some() {
            loop {
                if let Err(e) = email::send_all().await {
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
        .layer(middleware::from_fn_with_state((), auth_layer));

    let public_routes = Router::new()
        .route("/", get(home::view))
        .route("/nav", get(navigation::navigation))
        .route("/login", get(accounts::login_form).post(accounts::login))
        .route("/logout", get(accounts::logout))
        .route("/style.css", get(style::style))
        .nest("/event", events::public_router())
        .nest("/account", accounts::public_router())
        .nest("/shift", shift::public_router())
        .nest("/worker", worker::public_router())
        .nest("/email", email::public_router());

    // App
    let mid = ServiceBuilder::new()
        .layer(middleware::from_fn(html_wrapper))
        .layer(middleware::from_fn(session::session_provider))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // debug!(?request);
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);
                let uri = request.uri();

                let session = request.extensions().get::<Session>();
                let session_id = session.as_ref().map(|s| s.id()).map(|i| i.to_string());
                let user_id = session
                    .as_ref()
                    .and_then(|s| s.user_id())
                    .map(|u| u.to_string());

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    otel.name = matched_path,
                    otel.kind = "server",
                    session.id = session_id,
                    user.id = user_id,
                    %uri,
                )
            }),
        );

    let app = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .fallback(get_static_files)
        .layer(mid);

    let port = config().website.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::debug!("listening on {}", addr);

    match config().tls_config().cloned() {
        Some(cfg) => {
            axum_server::bind_rustls(addr, cfg)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        None => axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .unwrap(),
    };
    Ok(())
}

async fn auth_layer<B>(
    session: Session,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response<BoxBody>, AppError> {
    if !session.is_authenticated() {
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
