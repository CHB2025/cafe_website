use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    content: String,
}

pub async fn index() -> Html<String> {
    let body = Index {
        content: "Content".to_owned(),
    }
    .render()
    .expect("Valid template");
    Html(body)
}
