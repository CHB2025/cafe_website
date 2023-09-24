use askama::Template;

use crate::models::User;

#[derive(Template)]
#[template(path = "navigation.html")]
pub struct Nav {
    left: Vec<(String, String)>,
    right: Vec<(String, String)>,
}

pub async fn navigation(user: Option<User>) -> Nav {
    let (left, right) = if user.is_none() {
        (vec![], vec![("Log In".to_string(), "/login".to_string())])
    } else {
        (
            vec![("Events".to_string(), "/event/list".to_string())],
            vec![("Log Out".to_string(), "/logout".to_string())],
        )
    };
    Nav { left, right }
}
