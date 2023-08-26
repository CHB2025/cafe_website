use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrdinalPaginatedQuery {
    #[serde(default)]
    pub order_by: EventOrderBy,
    #[serde(default)]
    pub order_dir: OrderDirection,
    #[serde(default = "default_take")]
    pub take: i64,
    #[serde(default)]
    pub skip: i64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum OrderDirection {
    #[default]
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

fn default_take() -> i64 {
    10
}

impl fmt::Display for OrderDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            OrderDirection::Asc => "asc",
            OrderDirection::Desc => "desc",
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventOrderBy {
    #[default]
    Name,
    StartDate,
    EndDate,
}

impl fmt::Display for EventOrderBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Name => "name",
            Self::StartDate => "start_date",
            Self::EndDate => "end_date",
        };
        write!(f, "{}", str)
    }
}
