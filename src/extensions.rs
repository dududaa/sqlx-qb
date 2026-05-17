use crate::modifiers::{FilterJoiner, FilterOperator, QueryFilter};
use crate::value::QbValue;

pub fn eq<'q>(key: &'q str, value: impl Into<QbValue<'q>>) -> QueryFilter<'q> {
    QueryFilter::from((key, value)).with_op(FilterOperator::Eq)
}

pub fn and<'q>(value: impl Into<QueryFilter<'q>>) -> QueryFilter<'q> {
    value.into().with_joiner(FilterJoiner::And)
}

pub fn or<'q>(value: impl Into<QueryFilter<'q>>) -> QueryFilter<'q> {
    value.into().with_joiner(FilterJoiner::Or)
}
