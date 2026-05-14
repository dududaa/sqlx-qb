use crate::extension::QueryExt;
use crate::{QueryCommand, QuerySelectCommand, QuerySet, SqlxQb};

/// A query that returns ALL fields of the model.
pub fn select_query<'q>(table_name: &'q str, filters: QueryExt<'q>) -> SqlxQb<'q> {
    SqlxQb::new(
        QueryCommand::Select(QuerySelectCommand::SelectAll, table_name),
        filters,
    )
}

/// A query that returns provided fields of the return model.
pub fn select_fields_query<'q>(
    table_name: &'q str,
    fields: Vec<&'q str>,
    filters: QueryExt<'q>,
) -> SqlxQb<'q> {
    SqlxQb::new(
        QueryCommand::Select(QuerySelectCommand::SelectFields(fields), table_name),
        filters,
    )
}

pub fn delete_query<'q>(table_name: &'q str, filters: QueryExt<'q>) -> SqlxQb<'q> {
    SqlxQb::new(QueryCommand::Delete(table_name), filters)
}

pub fn update_query<'q>(
    table_name: &'q str,
    set: QuerySet<'q>,
    filters: QueryExt<'q>,
) -> SqlxQb<'q> {
    SqlxQb::new(QueryCommand::Update(table_name, set), filters)
}

pub mod extension {
    use crate::extension::{FilterJoiner, FilterOperator, QueryFilter};
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
}
