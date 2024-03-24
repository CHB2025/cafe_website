use askama::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, Template)]
#[template(path = "worker/view.html")]
pub struct Worker {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub name_first: String,
    pub name_last: String,
}
