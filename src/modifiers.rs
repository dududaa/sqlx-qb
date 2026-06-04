use crate::extensions::{and, or};
use std::fmt::{Display, Formatter};

pub struct Modifiers<'q> {
    filters: Vec<QueryFilter<'q>>,
    limit: Option<usize>,
    sort_by: Option<QuerySort<'q>>,
}

impl<'q> Modifiers<'q> {
    pub fn new() -> Self {
        Self {
            filters: vec![],
            limit: None,
            sort_by: None,
        }
    }

    /// Creates a `WHERE` clause.
    pub fn with_filter(mut self, filter: impl Into<QueryFilter<'q>>) -> Self {
        self.filters.push(filter.into());
        self
    }

    /// Appends a filter with `AND` join.
    pub fn and(mut self, filter: impl Into<QueryFilter<'q>>) -> Self {
        self.filters.push(and(filter));
        self
    }

    /// Appends a filter with `OR` join.
    pub fn or(mut self, filter: impl Into<QueryFilter<'q>>) -> Self {
        self.filters.push(or(filter));
        self
    }

    pub fn filters(&self) -> &[QueryFilter<'q>] {
        &self.filters
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_sort(mut self, sort_by: QuerySort<'q>) -> Self {
        self.sort_by = Some(sort_by);
        self
    }

    pub fn sql_str(&self, arg_offset: &usize) -> String {
        let clauses: Vec<String> = self
            .filters()
            .iter()
            .enumerate()
            .map(|(i, filter)| {
                let join = match filter.joiner {
                    Some(FilterJoiner::And) => "AND",
                    Some(FilterJoiner::Or) => "OR",
                    None => "",
                };

                format!(
                    "{} {} {} ${}",
                    join,
                    filter.key,
                    filter.operator,
                    i + arg_offset
                )
            })
            .collect();

        let clauses = if !clauses.is_empty() {
            let s = clauses.join(" ");
            format!(" WHERE {}", s.trim())
        } else {
            String::new()
        };

        let limit = self
            .limit
            .map(|limit| format!(" LIMIT {limit}"))
            .unwrap_or_default();

        let sort_by = self
            .sort_by
            .as_ref()
            .map(|s| format!(" ORDER BY {} {}", s.columns.join(","), s.dir))
            .unwrap_or_default();

        format!("{clauses}{sort_by}{limit}")
    }
}

impl<'q> Default for Modifiers<'q> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct QueryFilter<'q> {
    key: &'q str,
    value: String,
    joiner: Option<FilterJoiner>,
    operator: FilterOperator,
}

impl<'q> QueryFilter<'q> {
    pub fn key(&self) -> &str {
        &self.key
    }
    
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn with_op(mut self, value: FilterOperator) -> Self {
        self.operator = value;
        self
    }

    pub fn with_joiner(mut self, value: FilterJoiner) -> Self {
        self.joiner = Some(value);
        self
    }
}

pub enum FilterJoiner {
    And,
    Or,
}

pub enum FilterOperator {
    Eq,
    Gt,
}

impl Display for FilterOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use FilterOperator::*;

        match self {
            Eq => write!(f, "="),
            Gt => write!(f, ">"),
        }
    }
}

impl<'q> QueryFilter<'q> {
    fn new(key: &'q str, value: impl Into<String>) -> Self {
        QueryFilter {
            key,
            value: value.into(),
            joiner: None,
            operator: FilterOperator::Eq,
        }
    }
}

impl<'q, T: Display> From<(&'q str, T)> for QueryFilter<'q>
{
    fn from((k, v): (&'q str, T)) -> Self {
        QueryFilter::new(k, v.to_string())
    }
}

pub struct QuerySort<'q> {
    columns: Vec<&'q str>,
    dir: QuerySortDir,
}

impl<'q> QuerySort<'q> {
    pub fn new(columns: Vec<&'q str>, dir: QuerySortDir) -> Self {
        QuerySort { columns, dir }
    }
}

pub enum QuerySortDir {
    ASC,
    DESC,
}

impl Display for QuerySortDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            QuerySortDir::ASC => "ASC",
            QuerySortDir::DESC => "DESC",
        };

        write!(f, "{}", s)
    }
}

#[macro_export]
macro_rules! query_sort {
    ( $dir:expr, $( $column:literal ),* $(,)? ) => {{
        QuerySort::new(vec![$( $column, )*], $dir)
    }};
}
