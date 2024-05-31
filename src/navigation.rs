use askama::Template;

use crate::session::Session;

#[derive(Template)]
#[template(path = "navigation.html")]
pub struct Nav {
    left: Vec<(&'static str, &'static str)>,
    right: Vec<(&'static str, &'static str)>,
}

pub async fn navigation(session: Session) -> Nav {
    let (left, right) = if !session.is_authenticated() {
        (vec![], vec![("Log In", "/login")])
    } else {
        (
            vec![
                ("Events", "/event/list"),
                ("Workers", "/worker/list"),
                ("Emails", "/email/list"),
                ("Users", "/account/manage"),
            ],
            vec![("Log Out", "/logout")],
        )
    };
    Nav { left, right }
}
