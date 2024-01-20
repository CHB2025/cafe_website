use askama_axum::IntoResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct Redirect {
    path: String,
    target: String,
}

impl Redirect {
    pub fn to(path: String) -> Self {
        Redirect {
            path,
            target: "#content".to_owned(),
        }
    }

    pub fn targeted(path: String, target: String) -> Self {
        Redirect { path, target }
    }
}

impl IntoResponse for Redirect {
    fn into_response(self) -> askama_axum::Response {
        let headers = [("HX-Location", serde_json::to_string(&self).expect("Valid"))];

        // Pushes to history even if target is not content. Not ideal
        (headers, ()).into_response()
    }
}
