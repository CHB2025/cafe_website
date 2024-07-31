use std::fmt::Display;
use askama::Template;

#[derive(Template)]
#[template(path = "printable.html")]
pub struct Printable<T: Display> {
    content: T
}

impl<T: Display> Printable<T> {
    pub fn new(content: T) -> Self {
        Self {
            content
        }
    }
}
