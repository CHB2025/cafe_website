use askama_axum::IntoResponse;

pub struct Redirect {
    path: String,
}

impl Redirect {
    pub fn to(path: String) -> Self {
        Redirect { path }
    }
}

impl IntoResponse for Redirect {
    fn into_response(self) -> askama_axum::Response {
        let headers = [(
            "HX-Location",
            format!(r##"{{"path":"{}","target":"#content"}}"##, self.path),
        )];

        (headers, ()).into_response()
    }
}
