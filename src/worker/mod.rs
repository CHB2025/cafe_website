use axum::{routing::get, Router};

use crate::app_state::AppState;

use self::list::worker_list;

mod list;

pub fn protected_router() -> Router<AppState> {
    Router::new().route("/list", get(worker_list))
}
