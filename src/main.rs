use app_state::AppState;
use axum::{
    body::{boxed, Body, BoxBody, Bytes, HttpBody},
    extract::State,
    http::{Request, Response, StatusCode, Uri},
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use tokio::{fs, io::AsyncReadExt};
use tower::ServiceExt;
use tower_http::services::ServeDir;

mod app_state;
pub mod models;
mod routes;
pub mod schema;
pub(crate) mod utils;

#[tokio::main]
async fn main() {
    let mut index = fs::File::open("public/index.html").await.unwrap();
    let mut ind_html = String::new();
    index.read_to_string(&mut ind_html).await.unwrap();

    let app = Router::new()
        .route("/", get(|| async { Html("<p>Hello World</p>") }))
        .route("/signup", post(routes::create_account::signup))
        .with_state(AppState::init())
        .fallback(file_handler)
        .layer(middleware::from_fn_with_state(ind_html, html_wrapper));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn html_wrapper<B>(
    State(wrapper): State<String>,
    request: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    let from_htmx = request.headers().contains_key("HX-Request");
    let is_index = request.uri() == "/index.html" || request.uri() == "/index";
    let response = next.run(request).await;
    let is_html = response
        .headers()
        .get("content-type")
        .is_some_and(|ct| ct.as_bytes().starts_with(b"text/html"));

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

async fn file_handler(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, Html<&'static str>)> {
    let res = get_static_files(uri.clone()).await?;

    if res.status() == StatusCode::NOT_FOUND {
        let uri_html = format!("{}.html", uri).parse().map_err(utils::ise)?;
        get_static_files(uri_html).await
    } else {
        Ok(res)
    }
}

async fn get_static_files(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, Html<&'static str>)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    match ServeDir::new("./public").oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("Something went wrong"),
        )),
    }
}
