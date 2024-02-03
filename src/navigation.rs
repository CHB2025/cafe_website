use askama::Template;

use crate::models::User;

#[derive(Template)]
#[template(path = "navigation.html")]
pub struct Nav {
    left: Vec<(&'static str, &'static str)>,
    right: Vec<(&'static str, &'static str)>,
}

pub async fn navigation(user: Option<User>) -> Nav {
    let (left, right) = if user.is_none() {
        (vec![], vec![("Log In", "/login")])
    } else {
        (
            vec![
                ("Events", "/event/list"),
                ("Emails", "/email/list"),
                ("Users", "/account/manage"),
            ],
            vec![("Log Out", "/logout")],
        )
    };
    Nav { left, right }
}
