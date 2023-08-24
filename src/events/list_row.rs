use askama::Template;

use crate::models::Event;

#[derive(Template)]
#[template(path = "events/list_row.html")]
pub struct EventListRowTemplate {
    pub event: Event,
}

#[derive(Template)]
#[template(path = "events/edit_list_row.html")]
pub struct EditEventListRowTemplate {
    pub event: Event,
}
