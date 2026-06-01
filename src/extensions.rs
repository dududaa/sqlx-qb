use std::fmt::Display;
use crate::modifiers::{FilterJoiner, FilterOperator, QueryFilter};

pub fn eq(key: &'_ str, value: impl Display) -> QueryFilter<'_> {
    QueryFilter::from((key, value)).with_op(FilterOperator::Eq)
}

pub fn and<'q>(value: impl Into<QueryFilter<'q>>) -> QueryFilter<'q> {
    value.into().with_joiner(FilterJoiner::And)
}

pub fn or<'q>(value: impl Into<QueryFilter<'q>>) -> QueryFilter<'q> {
    value.into().with_joiner(FilterJoiner::Or)
}
