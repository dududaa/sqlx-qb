pub mod extensions;
mod map;
pub mod model;
pub mod modifiers;
mod query;
pub mod value;
pub mod prelude {
    pub use crate::extensions::*;
    pub use crate::map::*;
    pub use crate::model::*;
    pub use crate::modifiers::*;
    pub use crate::query_map;
    pub use crate::query_sort;
    pub use crate::QB;
}

#[cfg(feature = "postgres")]
type QbEngine = Postgres;

#[cfg(feature = "mysql")]
type QbEngine = MySql;

#[cfg(feature = "sqlite")]
type QbEngine = Sqlite;

#[cfg(feature = "any")]
type QbEngine = sqlx::Any;

#[cfg(feature = "postgres")]
type QbResult = PgQueryResult;

#[cfg(feature = "mysql")]
type QbResult = MySqlQueryResult;

#[cfg(feature = "sqlite")]
type QbResult = SqliteQueryResult;

#[cfg(feature = "any")]
type QbResult = AnyQueryResult;

use crate::model::Model;
use crate::query::{Query, QueryAs, QueryScalar, QueryWrapper};
use map::QueryMap;
use modifiers::QueryModifiers;
use sqlx::postgres::PgQueryResult;
use sqlx::{Database, Decode, Encode, FromRow, Pool, Postgres, Type};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

pub struct QB<'q, M: Model> {
    inner: SqlxQb<'q>,
    _model: PhantomData<M>,
}

impl<'q, M: Model> QB<'q, M> {
    pub fn new() -> Self {
        Self {
            inner: SqlxQb::default(),
            _model: PhantomData::default(),
        }
    }

    pub fn sql_str(&self) -> String {
        self.inner.sql_str()
    }

    pub fn insert(self, map: QueryMap<'q>) -> Self {
        self.with_command(QueryCommand::Insert(M::TABLE_NAME, map))
    }

    pub fn select(self) -> Self {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::SelectAll,
            M::TABLE_NAME,
        ))
    }

    pub fn select_fields(self, fields: Vec<&'q str>) -> Self {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::SelectFields(fields),
            M::TABLE_NAME,
        ))
    }

    pub fn update(self, set: QueryMap<'q>) -> Self {
        self.with_command(QueryCommand::Update(M::TABLE_NAME, set))
    }

    pub fn delete(self) -> Self {
        self.with_command(QueryCommand::Delete(M::TABLE_NAME))
    }

    pub fn with_modifiers(mut self, modifiers: QueryModifiers<'q>) -> Self {
        self.inner.modifiers = modifiers;
        self
    }

    pub fn reset_modifiers(mut self) -> Self {
        self.inner.modifiers = QueryModifiers::default();
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
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        self.inner.fetch_fields_one(db_pool).await
    }

    pub async fn fetch_fields_all<R>(&self, db_pool: &DbPool) -> Result<Vec<R>, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        self.inner.fetch_fields_all(db_pool).await
    }

    pub async fn execute(&self, db_pool: &DbPool) -> Result<QbResult, sqlx::Error> {
        self.inner.execute(db_pool).await
    }
}

pub type DbPool = Pool<QbEngine>;

struct SqlxQb<'q> {
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

        if let QueryCommand::Insert(_, map) = &self.cmd {
            for value in map.inner().values() {
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
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        let sql = self.sql_str();
        let query = QueryAs::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_one(db_pool).await
    }

    pub async fn fetch_fields_all<R>(&self, db_pool: &DbPool) -> Result<Vec<R>, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        let sql = self.sql_str();
        let query = QueryAs::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_all(db_pool).await
    }

    pub async fn execute(&self, db_pool: &DbPool) -> Result<QbResult, sqlx::Error> {
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
    Insert(&'q str, QueryMap<'q>),
    Select(QuerySelectCommand<'q>, &'q str),
    Update(&'q str, QueryMap<'q>),
    Delete(&'q str),
    Null,
}

impl<'q> Display for QueryCommand<'q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cmd = match self {
            QueryCommand::Insert(table_name, map) => {
                let columns = map
                    .inner()
                    .iter()
                    .map(|(col, _)| col.to_string())
                    .collect::<Vec<_>>();
                let values = map
                    .inner()
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("${}", i + 1))
                    .collect::<Vec<_>>();

                format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    table_name,
                    columns.join(", "),
                    values.join(", ")
                )
            }
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;
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

        let query = QB::<TestUserModel>::new()
            .select()
            .with_modifiers(modifiers);

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

        let set = query_map! {
          "name": "Demo User",
          "age": 34
        };

        let query = QB::<TestUserModel>::new()
            .update(set)
            .with_modifiers(modifiers);
        assert_eq!(
            query.sql_str(),
            "UPDATE users SET age = $1, name = $2 WHERE id = $3 AND business_id = $4 OR pid = $5"
        );
    }

    #[test]
    fn test_insert_query_sql_str() {
        let map = query_map! {
          "name": "Demo User",
          "age": 34
        };

        let query = QB::<TestUserModel>::new().insert(map);
        assert_eq!(
            query.sql_str(),
            "INSERT INTO users (age, name) VALUES ($1, $2)"
        )
    }

    #[test]
    fn test_order_by() {
        let modifiers =
            QueryModifiers::new().with_sort(query_sort!(QuerySortDir::DESC, "created_at"));
        let query = QB::<TestUserModel>::new()
            .select()
            .with_modifiers(modifiers);

        assert_eq!(
            query.sql_str(),
            "SELECT * FROM users ORDER BY created_at DESC"
        );
    }
}
