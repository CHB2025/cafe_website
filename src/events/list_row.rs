use askama::Template;

use crate::models::Event;

#[derive(Template)]
#[template(path = "events/list-row.html")]
pub struct EventListRowTemplate {
    pub event: Event,
}

#[derive(Template)]
#[template(path = "events/edit-list-row.html")]
pub struct EditEventListRowTemplate {
    pub event: Event,
}
