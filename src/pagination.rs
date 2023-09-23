use std::{fmt, ops};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct PaginatedQuery<O> {
    #[serde(default)]
    pub order_by: O,
    #[serde(default)]
    pub order_dir: OrderDirection,
    #[serde(default = "default_take")]
    pub take: i64,
    #[serde(default)]
    pub skip: i64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Hash)]
pub enum OrderDirection {
    #[default]
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

impl ops::Not for OrderDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
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

impl<O: Serialize> fmt::Display for PaginatedQuery<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_urlencoded::to_string(self).unwrap_or("".to_owned());
        write!(f, "{}", s)
    }
}

impl<O> PaginatedQuery<O> {
    pub fn next(mut self) -> Self {
        self.skip += self.take;
        self
    }

    pub fn previous(mut self) -> Self {
        self.skip = (self.skip - self.take).min(0);
        self
    }
}

impl<O> PaginatedQuery<O>
where
    O: PartialEq,
{
    pub fn with_order(mut self, order: O) -> Self {
        if self.order_by == order {
            self.order_dir = !self.order_dir;
        } else {
            self.order_by = order;
        }
        self
    }
}
