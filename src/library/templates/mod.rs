use askama::Template;

#[derive(Template)]
#[template(path = "card.html")]
pub struct Card<T: Template> {
    pub class: Option<&'static str>,
    pub title: String,
    pub child: T,
    pub show_x: bool,
}

impl<T: Template> Card<T> {
    pub fn modal(title: String, child: T) -> Self {
        Self {
            class: Some("w-fit"),
            title,
            child,
            show_x: true,
        }
    }
}
