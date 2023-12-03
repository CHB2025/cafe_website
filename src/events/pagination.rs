use std::fmt;

use serde::{Deserialize, Serialize};

pub use cafe_website::PaginatedQuery;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventOrderBy {
    #[default]
    Name,
}

impl fmt::Display for EventOrderBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Name => "name",
        };
        write!(f, "{}", str)
    }
}
