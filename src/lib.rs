mod extensions;
mod map;
mod model;
mod modifiers;
mod query;
mod value;
pub mod prelude {
    pub use crate::extensions::*;
    pub use crate::map::*;
    pub use crate::model::*;
    pub use crate::modifiers::*;
    pub use crate::query_map;
    pub use crate::query_sort;
    pub use crate::QB;
    pub use qb_macro::QbModel;
    pub use sqlx::FromRow;
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
use sqlx::{Database, Decode, Encode, FromRow, Pool, Type};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;

#[cfg(feature = "postgres")]
use sqlx::postgres::{PgQueryResult, Postgres};

#[cfg(feature = "mysql")]
use sqlx::mysql::{MySql, MySqlQueryResult};

#[cfg(feature = "sqlite")]
use sqlx::sqlite::{Sqlite, SqliteQueryResult};

#[cfg(feature = "any")]
use sqlx::any::{Any, AnyQueryResult};

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

    pub async fn insert(
        &mut self,
        map: QueryMap<'q>,
        db_pool: &DbPool,
    ) -> Result<QbResult, sqlx::Error> {
        self.with_command(QueryCommand::Insert(M::TABLE_NAME, map));
        let modifiers = self.modifiers;
        self.reset_modifiers();

        let result = M::insert(self, db_pool).await;
        if let Some(modifiers) = modifiers {
            self.inner.set_modifiers(modifiers);
        }

        result
    }

    pub async fn select(&mut self, db_pool: &DbPool) -> Result<M, sqlx::Error> {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::WildCard,
            M::TABLE_NAME,
        ));

        M::select(&self, db_pool).await
    }

    pub async fn select_all(&mut self, db_pool: &DbPool) -> Result<Vec<M>, sqlx::Error> {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::WildCard,
            M::TABLE_NAME,
        ));

        M::select_all(self, db_pool).await
    }

    pub async fn select_fields<R>(
        &mut self,
        fields: Vec<&'q str>,
        db_pool: &DbPool,
    ) -> Result<R, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::Fields(fields),
            M::TABLE_NAME,
        ));

        M::select_fields(self, db_pool).await
    }

    pub async fn select_fields_all<R>(
        &mut self,
        fields: Vec<&'q str>,
        db_pool: &DbPool,
    ) -> Result<Vec<R>, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::Fields(fields),
            M::TABLE_NAME,
        ));

        M::select_fields_all(self, db_pool).await
    }

    pub async fn update(
        &mut self,
        set: QueryMap<'q>,
        db_pool: &DbPool,
    ) -> Result<QbResult, sqlx::Error> {
        self.with_command(QueryCommand::Update(M::TABLE_NAME, set));
        M::update(self, db_pool).await
    }

    pub async fn delete(&mut self, db_pool: &DbPool) -> Result<QbResult, sqlx::Error> {
        self.with_command(QueryCommand::Delete(M::TABLE_NAME));
        M::delete(self, db_pool).await
    }

    pub fn with_modifiers(mut self, modifiers: &'q QueryModifiers<'q>) -> Self {
        self.inner.modifiers = Some(modifiers);
        self
    }

    pub fn reset_modifiers(&mut self) {
        self.inner.modifiers = None;
    }

    fn with_command(&mut self, cmd: QueryCommand<'q>) {
        self.inner.cmd = cmd;
    }
}

impl<'q, M: Model> Deref for QB<'q, M> {
    type Target = SqlxQb<'q>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub type DbPool = Pool<QbEngine>;

pub struct SqlxQb<'q> {
    cmd: QueryCommand<'q>,
    modifiers: Option<&'q QueryModifiers<'q>>,
}

impl<'q> SqlxQb<'q> {
    fn sql_str(&self) -> String {
        let mut arg_offset = 1;
        if let QueryCommand::Update(_, set) = &self.cmd {
            arg_offset += set.inner().len();
        }

        let builder_sql = self
            .modifiers
            .map_or(String::new(), |modifiers| modifiers.sql_str(&arg_offset));

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

        if let Some(modifiers) = self.modifiers {
            for clause in modifiers.filters() {
                query = clause.value().bind(query);
            }
        }

        query
    }

    fn set_modifiers(&mut self, modifiers: &'q QueryModifiers<'q>) {
        self.modifiers = Some(modifiers);
    }

    pub(crate) async fn fetch_one<M: Model>(&self, db_pool: &DbPool) -> Result<M, sqlx::Error> {
        let sql = self.sql_str();
        let query = QueryAs::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_one(db_pool).await
    }

    pub(crate) async fn fetch_all<M: Model>(
        &self,
        db_pool: &DbPool,
    ) -> Result<Vec<M>, sqlx::Error> {
        let sql = self.sql_str();
        let query = QueryAs::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_all(db_pool).await
    }

    pub(crate) async fn fetch_scalar_one<R>(&self, db_pool: &DbPool) -> Result<R, sqlx::Error>
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

    pub(crate) async fn fetch_scalar_all<R>(&self, db_pool: &DbPool) -> Result<Vec<R>, sqlx::Error>
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

    pub(crate) async fn fetch_fields_one<R>(&self, db_pool: &DbPool) -> Result<R, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        let sql = self.sql_str();
        let query = QueryAs::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_one(db_pool).await
    }

    pub(crate) async fn fetch_fields_all<R>(&self, db_pool: &DbPool) -> Result<Vec<R>, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        let sql = self.sql_str();
        let query = QueryAs::new(&sql);
        let query = self.bind_values(query).into_inner();

        query.fetch_all(db_pool).await
    }

    pub(crate) async fn execute(&self, db_pool: &DbPool) -> Result<QbResult, sqlx::Error> {
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
            modifiers: None,
        }
    }
}

enum QuerySelectCommand<'q> {
    WildCard,
    Fields(Vec<&'q str>),
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
                QuerySelectCommand::WildCard => format!("SELECT * FROM {}", table_name),
                QuerySelectCommand::Fields(fields) => {
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
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::{FromRow, SqlitePool};
    use std::str::FromStr;
    use uuid::Uuid;

    #[derive(QbModel, FromRow)]
    #[model(table_name = "users")]
    struct TestUserModel {}

    async fn pool() -> SqlitePool {
        let connection_options = SqliteConnectOptions::from_str("file::memory:?cache=shared")
            .unwrap()
            .create_if_missing(true);

        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(connection_options)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_select_query_sql_str() {
        let pool = pool().await;
        let modifiers = QueryModifiers::new()
            .with_filter(("id", 32))
            .and(eq("business_id", 32))
            .or(eq("pid", "some-pid"))
            .with_limit(1);

        let mut qb = QB::<TestUserModel>::new().with_modifiers(&modifiers);
        qb.select(&pool).await.ok();

        assert_eq!(
            qb.sql_str(),
            "SELECT * FROM users WHERE id = $1 AND business_id = $2 OR pid = $3 LIMIT 1"
                .to_string()
        );
    }

    #[tokio::test]
    async fn test_update_query_sql_str() {
        let pool = pool().await;
        let modifiers = QueryModifiers::new()
            .with_filter(("id", 32))
            .and(eq("business_id", 32))
            .or(eq("pid", Uuid::new_v4()));

        let set = query_map! {
          "name": "Demo User",
          "age": 34
        };

        let mut qb = QB::<TestUserModel>::new().with_modifiers(&modifiers);
        qb.update(set, &pool).await.ok();

        assert_eq!(
            qb.sql_str(),
            "UPDATE users SET age = $1, name = $2 WHERE id = $3 AND business_id = $4 OR pid = $5"
        );
    }

    #[tokio::test]
    async fn test_insert_query_sql_str() {
        let pool = pool().await;
        let map = query_map! {
          "name": "Demo User",
          "age": 34
        };

        let mut qb = QB::<TestUserModel>::new();
        qb.insert(map, &pool).await.ok();

        assert_eq!(
            qb.sql_str(),
            "INSERT INTO users (age, name) VALUES ($1, $2)"
        )
    }

    #[tokio::test]
    async fn test_order_by() {
        let pool = pool().await;
        let modifiers =
            QueryModifiers::new().with_sort(query_sort!(QuerySortDir::DESC, "created_at"));

        let mut qb = QB::<TestUserModel>::new().with_modifiers(&modifiers);
        qb.select(&pool).await.ok();

        assert_eq!(qb.sql_str(), "SELECT * FROM users ORDER BY created_at DESC");
    }
}
