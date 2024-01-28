use askama::Template;

#[derive(Template, Clone)]
#[template(path = "p_controls.html")]
pub struct PaginationControls {
    pub(super) class: Option<String>,
    pub(super) next_url: String,
    pub(super) prev_url: String,
    pub(super) next_disabled: bool,
    pub(super) prev_disabled: bool,
    pub(super) page: i64,
    pub(super) page_count: i64,
}
