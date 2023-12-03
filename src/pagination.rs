use std::{
    fmt::{self, Display},
    ops,
};

use serde::{Deserialize, Serialize};

/// A general-purpose struct to extract pagination query params.
/// WARNING: Do not use raw user input for the OrderBy type (eg. a String)
///          as it may open you to sql injection attacks.
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

    pub fn page(&self) -> i64 {
        self.skip / self.take + 1
    }

    pub fn page_count(&self, total_count: i64) -> i64 {
        total_count / self.take + (total_count % self.take > 0) as i64
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
            self.order_dir = OrderDirection::Asc;
        }
        self
    }
}

impl<O> PaginatedQuery<O>
where
    O: Display,
{
    pub fn sql(&self) -> String {
        format!(
            "ORDER BY {} {} LIMIT {} OFFSET {}",
            self.order_by, self.order_dir, self.take, self.skip
        )
    }
}
