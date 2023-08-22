use askama::Template;

#[derive(Template)]
#[template(path = "list.html")]
pub struct List<const N: usize, T: Template> {
    pub container_class: String,
    pub header: [String; N],
    pub rows: Vec<T>,

    pub order_by: String,
    pub order_dir: String,

    pub prev_disabled: bool,
    pub prev_url: String,
    pub next_disabled: bool,
    pub next_url: String,
    pub current_page: i64,
    pub page_count: i64,
}
