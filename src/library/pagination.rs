use std::{
    fmt::{self, Display},
    ops,
};

use serde::{Deserialize, Serialize};

pub use controls::PaginationControls;

mod controls;

/// A general-purpose struct to extract pagination query params.
/// The display implementation creates a url query string with the parameters.
/// To get the sql, use [PaginatedQuery::sql] instead.
///
/// WARNING: Do not use raw user input for the OrderBy type (eg. a String)
///          as it may open you to sql injection attacks.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct PaginatedQuery<O, const DS: i64 = { 10 }, const ASC: bool = { true }> {
    #[serde(default)]
    pub order_by: O,
    #[serde(default = "PaginatedQuery::<O, DS, ASC>::default_dir")]
    pub order_dir: OrderDirection,
    #[serde(default = "PaginatedQuery::<O, DS>::default_take")]
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

impl fmt::Display for OrderDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            OrderDirection::Asc => "asc",
            OrderDirection::Desc => "desc",
        };
        write!(f, "{}", str)
    }
}

impl<O: Serialize, const DS: i64, const ASC: bool> fmt::Display for PaginatedQuery<O, DS, ASC> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_urlencoded::to_string(self).unwrap_or("".to_owned());
        write!(f, "{}", s)
    }
}

impl<O, const DS: i64, const ASC: bool> PaginatedQuery<O, DS, ASC> {
    pub fn next(mut self) -> Self {
        self.skip += self.take;
        self
    }

    pub fn previous(mut self) -> Self {
        self.skip = (self.skip - self.take).max(0);
        self
    }

    pub fn page(&self) -> i64 {
        self.skip / self.take + 1
    }

    pub fn page_count(&self, total_count: i64) -> i64 {
        total_count / self.take + (total_count % self.take > 0) as i64
    }

    pub fn take(mut self, take: i64) -> Self {
        self.take = take;
        self
    }

    fn default_take() -> i64 {
        DS
    }

    fn default_dir() -> OrderDirection {
        if ASC {
            OrderDirection::Asc
        } else {
            OrderDirection::Desc
        }
    }
}

impl<O, const DS: i64, const ASC: bool> PaginatedQuery<O, DS, ASC>
where
    O: Serialize + Copy,
{
    pub fn controls(self, record_count: i64, url: String) -> PaginationControls {
        let page = self.page();
        let page_count = self.page_count(record_count);
        PaginationControls {
            class: None,
            next_url: format!("{url}{}", self.next()),
            prev_url: format!("{url}{}", self.previous()),
            prev_disabled: page == 1,
            next_disabled: page == page_count,
            page: self.page(),
            page_count,
        }
    }

    pub fn class_controls(
        self,
        record_count: i64,
        url: String,
        class: String,
    ) -> PaginationControls {
        let page = self.page();
        let page_count = self.page_count(record_count);
        PaginationControls {
            class: Some(class),
            next_url: format!("{url}{}", self.next()),
            prev_url: format!("{url}{}", self.previous()),
            prev_disabled: page == 1,
            next_disabled: page == page_count,
            page: self.page(),
            page_count,
        }
    }
}

impl<O, const DS: i64, const ASC: bool> PaginatedQuery<O, DS, ASC>
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

impl<O, const DS: i64, const ASC: bool> PaginatedQuery<O, DS, ASC>
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
