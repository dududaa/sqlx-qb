pub mod apis;
pub mod extension;
pub mod value;
pub mod model;

use crate::model::Model;
use extension::QueryExt;
use sqlx::postgres::{PgArguments, PgQueryResult, PgRow};
use sqlx::query::{Query, QueryAs, QueryScalar};
use sqlx::{FromRow, Pool, Postgres};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use value::QbValue;

pub use apis::{select_query, update_query};

pub type DbPool = Pool<Postgres>;

pub struct SqlxQb<'q> {
    cmd: QueryCommand<'q>,
    ext: QueryExt<'q>,
}

impl<'q> SqlxQb<'q> {
    fn new(cmd: QueryCommand<'q>, ext: QueryExt<'q>) -> SqlxQb<'q> {
        SqlxQb { cmd, ext }
    }

    fn sql_str(&self) -> String {
        let mut arg_offset = 1;
        if let QueryCommand::Update(_, set) = &self.cmd {
            arg_offset += set.inner().len();
        }

        let builder_sql = self.ext.sql_str(&arg_offset);
        format!("{}{}", self.cmd, builder_sql)
    }

    fn bind_values_on_query(
        &self,
        mut query: Query<'q, Postgres, PgArguments>,
    ) -> Query<'q, Postgres, PgArguments> {
        if let QueryCommand::Update(_, set) = &self.cmd {
            for (_, value) in set.inner() {
                query = value.clone().bind_to_query(query);
            }
        }

        for clause in self.ext.filters() {
            query = clause.value().bind_to_query(query);
        }

        query
    }

    fn bind_values_on_query_as<M>(
        &self,
        mut query: QueryAs<'q, Postgres, M, PgArguments>,
    ) -> QueryAs<'q, Postgres, M, PgArguments>
    where
        M: Sized + Send + Unpin + for<'r> FromRow<'r, PgRow>,
    {
        for clause in self.ext.filters() {
            query = clause.value().bind_to_query_as(query);
        }

        query
    }

    fn bind_values_on_query_scalar<R>(
        &self,
        mut query: QueryScalar<'q, Postgres, R, PgArguments>,
    ) -> QueryScalar<'q, Postgres, R, PgArguments> {
        for clause in self.ext.filters() {
            query = clause.value().bind_to_query_scalar(query);
        }

        query
    }

    pub async fn fetch_all<M: Model>(&self, db_pool: &DbPool) -> Result<Vec<M>, sqlx::Error> {
        let sql = self.sql_str();
        let query = self.bind_values_on_query_as(sqlx::query_as::<Postgres, M>(&sql));

        query.fetch_all(db_pool).await
    }

    pub async fn fetch_one<M: Model>(&self, db_pool: &DbPool) -> Result<M, sqlx::Error> {
        let sql = self.sql_str();
        let query = self.bind_values_on_query_as(sqlx::query_as::<Postgres, M>(&sql));

        query.fetch_one(db_pool).await
    }

    pub async fn fetch_scalar_one<R>(&self, db_pool: &DbPool) -> Result<R, sqlx::Error>
    where
        R: Send + Unpin,
        (R,): for<'r> FromRow<'r, PgRow>,
    {
        let sql = self.sql_str();
        let query = self.bind_values_on_query_scalar(sqlx::query_scalar::<Postgres, R>(&sql));

        query.fetch_one(db_pool).await
    }

    pub async fn fetch_fields_all<R>(&self, db_pool: &DbPool) -> Result<Vec<R>, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, PgRow>,
    {
        let sql = self.sql_str();
        let query = self.bind_values_on_query_as(sqlx::query_as::<Postgres, R>(&sql));

        query.fetch_all(db_pool).await
    }

    pub async fn execute(&self, db_pool: &DbPool) -> Result<PgQueryResult, sqlx::Error> {
        let sql = self.sql_str();
        let query = self.bind_values_on_query(sqlx::query::<Postgres>(&sql));

        query.execute(db_pool).await
    }
}

enum QuerySelectCommand<'q> {
    SelectAll,
    SelectFields(Vec<&'q str>),
}

enum QueryCommand<'q> {
    Select(QuerySelectCommand<'q>, &'q str),
    Update(&'q str, QuerySet<'q>),
    Delete(&'q str),
}

impl<'q> Display for QueryCommand<'q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cmd = match self {
            QueryCommand::Select(select, table_name) => match select {
                QuerySelectCommand::SelectAll => format!("SELECT * FROM {}", table_name),
                QuerySelectCommand::SelectFields(fields) => {
                    format!("SELECT {} FROM {}", fields.join(", "), table_name)
                }
            },
            QueryCommand::Delete(table_name) => format!("DELETE FROM {}", table_name),
            QueryCommand::Update(table_name, set) => {
                let ff = set
                    .inner()
                    .iter()
                    .enumerate()
                    .map(|(i, (col, _))| format!("{col} = ${}", i + 1))
                    .collect::<Vec<String>>()
                    .join(", ");

                format!("UPDATE {} SET {}", table_name, ff)
            }
        };

        write!(f, "{}", cmd)
    }
}

pub struct QuerySet<'q>(HashMap<&'q str, QbValue<'q>>);
impl<'q> QuerySet<'q> {
    pub fn new(key: &'q str, value: impl Into<QbValue<'q>>) -> Self {
        let mut map = HashMap::new();
        map.insert(key, value.into());

        QuerySet(map)
    }
    pub fn add(mut self, key: &'q str, value: impl Into<QbValue<'q>>) -> Self {
        self.0.insert(key, value.into());
        self
    }

    fn inner(&self) -> &HashMap<&'q str, QbValue<'q>> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apis::extension::eq;
    use crate::extension::{QuerySort, QuerySortDir};
    use crate::{select_query, update_query};
    use uuid::Uuid;

    #[test]
    fn test_select_query_sql_str() {
        let qb = QueryExt::new()
            .with_filter(("id", 32))
            .and(eq("business_id", 32))
            .or(eq("pid", Uuid::new_v4()))
            .with_limit(1);

        let query = select_query("users", qb);

        assert_eq!(
            query.sql_str(),
            "SELECT * FROM users WHERE id = $1 AND business_id = $2 OR pid = $3 LIMIT 1"
                .to_string()
        );
    }

    #[test]
    fn test_update_query_sql_str() {
        let qb = QueryExt::new()
            .with_filter(("id", 32))
            .and(eq("business_id", 32))
            .or(eq("pid", Uuid::new_v4()));

        let set = QuerySet::new("name", "Demo User").add("age", 34);
        let query = update_query("users", set, qb);

        assert_eq!(
            query.sql_str(),
            "UPDATE users SET age = $1, name = $2 WHERE id = $3 AND business_id = $4 OR pid = $5"
        );
    }

    #[test]
    fn test_order_by() {
        let qb_ext =
            QueryExt::new().with_sort(QuerySort::new(vec!["created_at"], QuerySortDir::DESC));
        let query = select_query("users", qb_ext);

        assert_eq!(
            query.sql_str(),
            "SELECT * FROM users ORDER BY created_at DESC"
        );
    }
}
