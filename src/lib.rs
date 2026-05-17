pub mod apis;
pub mod model;
pub mod modifiers;
mod query;
pub mod value;

#[cfg(feature = "postgres")]
type QbEngine = Postgres;

#[cfg(feature = "mysql")]
type QbEngine = MySql;

#[cfg(feature = "sqlite")]
type QbEngine = Sqlite;

#[cfg(feature = "any")]
type QbEngine = sqlx::Any;

use crate::model::Model;
use modifiers::QueryModifiers;
use sqlx::postgres::{PgQueryResult, PgRow};
use sqlx::{Database, Decode, Encode, FromRow, Pool, Postgres, Type};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use value::QbValue;

use crate::query::{Query, QueryAs, QueryScalar, QueryWrapper};

pub struct QB<'q, M: Model> {
    inner: SqlxQb<'q>,
    _model: PhantomData<M>,
}

impl<'q, M: Model> QB<'q, M> {
    fn new() -> Self {
        Self {
            inner: SqlxQb::default(),
            _model: PhantomData::default(),
        }
    }

    pub fn sql_str(&self) -> String {
        self.inner.sql_str()
    }

    pub fn select() -> Self {
        Self::new().with_command(QueryCommand::Select(
            QuerySelectCommand::SelectAll,
            M::TABLE_NAME,
        ))
    }

    pub fn select_fields(fields: Vec<&'q str>) -> Self {
        Self::new().with_command(QueryCommand::Select(
            QuerySelectCommand::SelectFields(fields),
            M::TABLE_NAME,
        ))
    }

    pub fn update(set: QuerySet<'q>) -> Self {
        Self::new().with_command(QueryCommand::Update(M::TABLE_NAME, set))
    }

    pub fn delete() -> Self {
        Self::new().with_command(QueryCommand::Delete(M::TABLE_NAME))
    }

    pub fn with_modifiers(mut self, modifiers: QueryModifiers<'q>) -> Self {
        self.inner.modifiers = modifiers;
        self
    }

    fn with_command(mut self, cmd: QueryCommand<'q>) -> Self {
        self.inner.cmd = cmd;
        self
    }

    pub async fn fetch(&self, db_pool: &DbPool) -> Result<M, sqlx::Error> {
        self.inner.fetch_one(db_pool).await
    }

    pub async fn fetch_all(&self, db_pool: &DbPool) -> Result<Vec<M>, sqlx::Error> {
        self.inner.fetch_all(db_pool).await
    }

    pub async fn fetch_scalar<R>(&self, db_pool: &DbPool) -> Result<R, sqlx::Error>
    where
        R: Send + Unpin,
        R: for<'r> Encode<'r, QbEngine> + for<'r> Decode<'r, QbEngine> + Type<QbEngine>,
        (R,): for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        self.inner.fetch_scalar_one(db_pool).await
    }

    pub async fn fetch_scalar_all<R>(&self, db_pool: &DbPool) -> Result<Vec<R>, sqlx::Error>
    where
        R: Send + Unpin,
        R: for<'r> Encode<'r, QbEngine> + for<'r> Decode<'r, QbEngine> + Type<QbEngine>,
        (R,): for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        self.inner.fetch_scalar_all(db_pool).await
    }

    pub async fn fetch_fields<R>(&self, db_pool: &DbPool) -> Result<R, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, PgRow>,
    {
        self.inner.fetch_fields_one(db_pool).await
    }

    pub async fn fetch_fields_all<R>(&self, db_pool: &DbPool) -> Result<Vec<R>, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, PgRow>,
    {
        self.inner.fetch_fields_all(db_pool).await
    }
}

pub type DbPool = Pool<QbEngine>;

pub struct SqlxQb<'q> {
    cmd: QueryCommand<'q>,
    modifiers: QueryModifiers<'q>,
}

impl<'q> SqlxQb<'q> {

    fn sql_str(&self) -> String {
        let mut arg_offset = 1;
        if let QueryCommand::Update(_, set) = &self.cmd {
            arg_offset += set.inner().len();
        }

        let builder_sql = self.modifiers.sql_str(&arg_offset);
        format!("{}{}", self.cmd, builder_sql)
    }

    fn bind_values<Q: QueryWrapper<'q>>(&self, mut query: Q) -> Q {
        if let QueryCommand::Update(_, set) = &self.cmd {
            for value in set.inner().values() {
                query = value.clone().bind(query);
            }
        }

        for clause in self.modifiers.filters() {
            query = clause.value().bind(query);
        }

        query
    }

    async fn fetch_all<M: Model>(&self, db_pool: &DbPool) -> Result<Vec<M>, sqlx::Error> {
        let sql = self.sql_str();
        let query = QueryAs::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_all(db_pool).await
    }

    async fn fetch_one<M: Model>(&self, db_pool: &DbPool) -> Result<M, sqlx::Error> {
        let sql = self.sql_str();
        let query = QueryAs::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_one(db_pool).await
    }

    async fn fetch_scalar_one<R>(&self, db_pool: &DbPool) -> Result<R, sqlx::Error>
    where
        R: Send + Unpin,
        R: for<'r> Encode<'r, QbEngine> + for<'r> Decode<'r, QbEngine> + Type<QbEngine>,
        (R,): for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        let sql = self.sql_str();
        let query = QueryScalar::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_one(db_pool).await
    }

    async fn fetch_scalar_all<R>(&self, db_pool: &DbPool) -> Result<Vec<R>, sqlx::Error>
    where
        R: Send + Unpin,
        R: for<'r> Encode<'r, QbEngine> + for<'r> Decode<'r, QbEngine> + Type<QbEngine>,
        (R,): for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        let sql = self.sql_str();
        let query = QueryScalar::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_all(db_pool).await
    }

    pub async fn fetch_fields_one<R>(&self, db_pool: &DbPool) -> Result<R, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, PgRow>,
    {
        let sql = self.sql_str();
        let query = QueryAs::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_one(db_pool).await
    }

    pub async fn fetch_fields_all<R>(&self, db_pool: &DbPool) -> Result<Vec<R>, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, PgRow>,
    {
        let sql = self.sql_str();
        let query = QueryAs::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_all(db_pool).await
    }

    pub async fn execute(&self, db_pool: &DbPool) -> Result<PgQueryResult, sqlx::Error> {
        let sql = self.sql_str();
        let query = Query::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.execute(db_pool).await
    }
}

impl<'q> Default for SqlxQb<'q> {
    fn default() -> Self {
        Self {
            cmd: QueryCommand::Null,
            modifiers: QueryModifiers::default(),
        }
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
    Null,
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
            QueryCommand::Null => "NULL".to_string(),
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
    use crate::modifiers::{QuerySort, QuerySortDir};
    use sqlx::FromRow;
    use uuid::Uuid;

    #[derive(FromRow)]
    struct TestUserModel {}

    impl Model for TestUserModel {
        const TABLE_NAME: &'static str = "users";
    }

    #[test]
    fn test_select_query_sql_str() {
        let modifiers = QueryModifiers::new()
            .with_filter(("id", 32))
            .and(eq("business_id", 32))
            .or(eq("pid", Uuid::new_v4()))
            .with_limit(1);

        let query = QB::<TestUserModel>::select().with_modifiers(modifiers);

        assert_eq!(
            query.sql_str(),
            "SELECT * FROM users WHERE id = $1 AND business_id = $2 OR pid = $3 LIMIT 1"
                .to_string()
        );
    }

    #[test]
    fn test_update_query_sql_str() {
        let modifiers = QueryModifiers::new()
            .with_filter(("id", 32))
            .and(eq("business_id", 32))
            .or(eq("pid", Uuid::new_v4()));

        let set = QuerySet::new("name", "Demo User").add("age", 34);
        let query = QB::<TestUserModel>::update(set).with_modifiers(modifiers);

        assert_eq!(
            query.sql_str(),
            "UPDATE users SET age = $1, name = $2 WHERE id = $3 AND business_id = $4 OR pid = $5"
        );
    }

    #[test]
    fn test_order_by() {
        let modifiers =
            QueryModifiers::new().with_sort(QuerySort::new(vec!["created_at"], QuerySortDir::DESC));
        let query = QB::<TestUserModel>::select().with_modifiers(modifiers);

        assert_eq!(
            query.sql_str(),
            "SELECT * FROM users ORDER BY created_at DESC"
        );
    }
}
