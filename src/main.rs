use std::env;

use axum::{
    body::{boxed, Body, BoxBody},
    http::{Request, Response, StatusCode, Uri},
    response::Html,
    routing::get,
    Router,
};
use diesel::{pg::PgConnection, prelude::*};
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub mod models;
pub mod schema;

#[tokio::main]
async fn main() {
    let db = establish_db_connection();

    let app = Router::new()
        .route("/", get(home))
        .route("/test", get(|| async { Html("<p>Hello test</p>") }))
        .fallback(get_static_files);
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn establish_db_connection() -> PgConnection {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect("Error connecting to the database")
}

async fn home() -> Result<Response<BoxBody>, (StatusCode, String)> {
    get_static_files(Uri::from_static("/index.html")).await
}

async fn get_static_files(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    match ServeDir::new("./public").oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}
