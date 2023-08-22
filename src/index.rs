use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {}
