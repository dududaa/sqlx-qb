mod extensions;
mod map;
mod model;
mod modifiers;
mod query;

pub mod prelude {
    pub use crate::extensions::*;
    pub use crate::map::*;
    pub use crate::model::*;
    pub use crate::modifiers::*;
    pub use crate::query_map;
    pub use crate::query_sort;
    pub use crate::QB;
    pub use qb_macro::Model;
    pub use sqlx::{Database, Executor, FromRow, IntoArguments};
    pub use std::future::Future;
}

use crate::model::Model;
use crate::query::{Query, QueryAs, QueryScalar};
use map::QueryMap;
use modifiers::QueryModifiers;
use sqlx::{Database, Decode, Encode, Error, Executor, FromRow, IntoArguments, Type};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

use crate::prelude::ModelInsert;
#[cfg(feature = "serde")]
use serde::Serialize;

pub struct QB<'q, DB, P>
where
    DB: Database,
    P: Executor<'q, Database = DB> + Clone,
{
    inner: SqlxQb<'q, DB, P>,
    table_name: Option<&'q str>,
}

impl<'q, DB, E> QB<'q, DB, E>
where
    DB: Database,
    E: Executor<'q, Database = DB> + Clone,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
    DB::Arguments: IntoArguments<DB>,
{
    pub fn new(pool: E) -> Self {
        Self {
            inner: SqlxQb::new(pool),
            table_name: None,
        }
    }

    pub fn pool(&'q self) -> &'q E {
        &self.pool
    }

    pub fn sql_str(&self) -> String {
        self.inner.sql_str()
    }

    pub fn with_table_name(mut self, table_name: &'q str) -> Self {
        self.table_name = Some(table_name);
        self
    }

    pub fn set_table_name(&mut self, table_name: &'q str) {
        self.table_name = Some(table_name);
    }

    pub fn table_name(&self) -> Option<&'q str> {
        self.table_name.clone()
    }

    #[cfg(not(feature = "serde"))]
    pub async fn insert<I: ModelInsert<'q>>(&'q mut self, map: &'q I) -> Result<(), Error> {
        map.insert(self).await
    }

    #[cfg(not(feature = "serde"))]
    pub async fn insert_returns<I: ModelInsert<'q>>(
        &'q mut self,
        map: &'q I,
        column: &'q str,
    ) -> Result<I::InsertReturns, Error>
    where
        I::InsertReturns:
            for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB> + Send + Unpin + 'q,
        (I::InsertReturns,): for<'r> FromRow<'r, DB::Row>,
    {
        map.insert_returns(self, column).await
    }

    #[cfg(feature = "serde")]
    pub async fn insert<M: Model, T: Serialize>(&'q mut self, value: &'q T) -> Result<(), Error> {
        let map = QueryMap::from_value(value)?;
        M::insert(self, map).await
    }

    #[cfg(feature = "serde")]
    /// Insert data and returns the specified `column`. Call this ONLY if your database supports RETURNING statement.
    pub async fn insert_returns<M: Model, T: Serialize>(
        &'q mut self,
        value: &'q T,
        column: &'q str,
    ) -> Result<M::InsertReturns, Error>
    where
        M::InsertReturns:
            for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB> + Send + Unpin + 'q,
        (M::InsertReturns,): for<'r> FromRow<'r, DB::Row>,
    {
        let map = QueryMap::from_value(value)?;
        M::insert_returns(self, map, column).await
    }

    pub async fn select<M>(&mut self) -> Result<M, Error>
    where
        M: Model<'q, DB, E> + for<'r> sqlx::FromRow<'r, DB::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::WildCard,
            M::TABLE_NAME,
        ));

        M::select(self).await
    }

    pub async fn select_all<M: Model<'q, DB, E> + for<'r> sqlx::FromRow<'r, DB::Row>>(
        &mut self,
    ) -> Result<Vec<M>, Error> {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::WildCard,
            M::TABLE_NAME,
        ));

        M::select_all(self).await
    }

    pub async fn select_fields<M, R>(&mut self, fields: impl Into<Vec<&'q str>>) -> Result<R, Error>
    where
        M: Model<'q, DB, E>,
        R: Send + Unpin + for<'r> FromRow<'r, DB::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::Fields(fields.into()),
            M::TABLE_NAME,
        ));

        M::select_fields::<M, R>(self).await
    }

    pub async fn select_fields_all<M, R>(
        &mut self,
        fields: impl Into<Vec<&'q str>>,
    ) -> Result<Vec<R>, Error>
    where
        M: Model<'q, DB, E>,
        R: Send + Unpin + for<'r> FromRow<'r, DB::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::Fields(fields.into()),
            M::TABLE_NAME,
        ));

        M::select_fields_all::<M, R>(self).await
    }

    pub async fn select_scalar<M, R>(&mut self, field: &'q str) -> Result<R, Error>
    where
        M: Model<'q, DB, E>,
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::Fields([field].into()),
            M::TABLE_NAME,
        ));

        M::select_scalar::<M, R>(self).await
    }

    pub async fn select_scalar_all<M, R>(&mut self, field: &'q str) -> Result<Vec<R>, Error>
    where
        M: Model<'q, DB, E>,
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::Fields([field].into()),
            M::TABLE_NAME,
        ));

        M::select_scalar_all::<M, R>(self).await
    }

    #[cfg(feature = "serde")]
    pub async fn update<M: Model, T: Serialize>(&mut self, value: &'q T) -> Result<(), Error> {
        let map = QueryMap::from_value(value)?;
        self.update_map::<M>(map).await
    }

    #[cfg(not(feature = "serde"))]
    pub async fn update<M: Model<'q, DB, E>>(&mut self, value: QueryMap) -> Result<(), Error> {
        self.update_map::<M>(value).await
    }

    pub async fn update_map<M: Model<'q, DB, E>>(&mut self, set: QueryMap) -> Result<(), Error> {
        self.with_command(QueryCommand::Update(M::TABLE_NAME, set));
        self.execute().await
    }

    pub async fn delete<M: Model<'q, DB, E>>(mut self) -> Result<(), Error> {
        self.with_command(QueryCommand::Delete(M::TABLE_NAME));
        self.execute().await
    }

    pub fn with_modifiers(mut self, modifiers: &'q QueryModifiers<'q>) -> Self {
        self.inner.modifiers = Some(modifiers);
        self
    }

    pub fn reset_modifiers(&mut self) {
        self.inner.modifiers = None;
    }

    fn with_command(&mut self, command: QueryCommand<'q>) {
        self.inner.cmd = command;
        self.collect_args();
    }
}

impl<'q, DB, P> Deref for QB<'q, DB, P>
where
    DB: Database,
    P: Executor<'q, Database = DB> + Clone,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
{
    type Target = SqlxQb<'q, DB, P>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'q, DB, P> DerefMut for QB<'q, DB, P>
where
    DB: Database,
    P: Executor<'q, Database = DB> + Clone,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct SqlxQb<'q, DB, P>
where
    DB: Database,
    P: Executor<'q, Database = DB> + Clone,
{
    cmd: QueryCommand<'q>,
    modifiers: Option<&'q QueryModifiers<'q>>,
    args: Vec<String>,
    pool: P,
}

impl<'q, DB, E> SqlxQb<'q, DB, E>
where
    DB: Database,
    E: Executor<'q, Database = DB> + Clone,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
{
    fn new(pool: E) -> Self {
        Self {
            cmd: QueryCommand::Null,
            modifiers: None,
            pool,
            args: Default::default(),
        }
    }

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

    fn collect_args(&mut self)
    where
        String: sqlx::Encode<'q, DB>,
        String: sqlx::Type<DB>,
    {
        if let QueryCommand::Update(_, set) = &self.cmd {
            for v in set.inner().values() {
                self.args.push(v.clone());
            }
        }

        if let QueryCommand::Insert { map, .. } = &self.cmd {
            for v in map.inner().values() {
                self.args.push(v.clone());
            }
        }

        if let Some(modifiers) = self.modifiers {
            for clause in modifiers.filters() {
                let parsed = clause.value().parse().unwrap_or_default();
                self.args.push(parsed);
            }
        }
    }

    pub fn set_modifiers(&mut self, modifiers: &'q QueryModifiers<'q>) {
        self.modifiers = Some(modifiers);
    }

    pub(crate) async fn fetch_one<M: Model<'q, DB, E> + for<'r> sqlx::FromRow<'r, DB::Row>>(
        &self,
    ) -> Result<M, Error>
    where
        DB::Arguments: IntoArguments<DB>,
    {
        let sql = self.sql_str();
        let args = self.args.clone();
        let query = QueryAs::new(sql, args);

        query.fetch_one(self.pool.clone()).await
    }

    pub(crate) async fn fetch_all<M>(&self) -> Result<Vec<M>, Error>
    where
        M: Model<'q, DB, E> + for<'r> sqlx::FromRow<'r, DB::Row>,
        DB::Arguments: IntoArguments<DB>,
    {
        let sql = self.sql_str();
        let args = self.args.clone();
        let query = QueryAs::new(sql, args);

        query.fetch_all(self.pool.clone()).await
    }

    pub(crate) async fn fetch_scalar_one<R>(&self) -> Result<R, Error>
    where
        DB::Arguments: IntoArguments<DB>,
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        let args = self.args.clone();
        let sql = self.sql_str();
        let query = QueryScalar::new(sql, args);

        query.fetch_one(self.pool.clone()).await
    }

    pub(crate) async fn fetch_scalar_all<R>(&self) -> Result<Vec<R>, Error>
    where
        DB::Arguments: IntoArguments<DB>,
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        let sql = self.sql_str();
        let args = self.args.clone();
        let query = QueryScalar::new(sql, args);

        query.fetch_all(self.pool.clone()).await
    }

    pub(crate) async fn fetch_fields_one<R>(&self) -> Result<R, Error>
    where
        DB::Arguments: IntoArguments<DB>,
        R: Send + Unpin + for<'r> FromRow<'r, DB::Row>,
    {
        let sql = self.sql_str();
        let args = self.args.clone();
        let query = QueryAs::new(sql, args);

        query.fetch_one(self.pool.clone()).await
    }

    pub(crate) async fn fetch_fields_all<R>(&self) -> Result<Vec<R>, Error>
    where
        DB::Arguments: IntoArguments<DB>,
        R: Send + Unpin + for<'r> FromRow<'r, DB::Row>,
    {
        let sql = self.sql_str();
        let args = self.args.clone();
        let query = QueryAs::new(sql, args);

        query.fetch_all(self.pool.clone()).await
    }

    pub async fn execute(&self) -> Result<(), Error>
    where
        DB::Arguments: IntoArguments<DB>,
    {
        let sql = self.sql_str();
        let args = self.args.clone();
        let query = Query::new(sql, args);

        query.execute(self.pool.clone()).await?;
        Ok(())
    }
}

enum QuerySelectCommand<'q> {
    WildCard,
    Fields(Vec<&'q str>),
}

enum QueryCommand<'q> {
    Insert {
        table_name: &'q str,
        map: QueryMap,
        /// The column the table should return after creating.
        returning: Option<&'q str>,
    },
    Select(QuerySelectCommand<'q>, &'q str),
    Update(&'q str, QueryMap),
    Delete(&'q str),
    Null,
}

impl<'q> Display for QueryCommand<'q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cmd = match self {
            QueryCommand::Insert {
                table_name,
                map,
                returning: returns,
            } => {
                let columns = map
                    .inner()
                    .keys()
                    .map(|col| col.to_string())
                    .collect::<Vec<_>>();

                let values = map
                    .inner()
                    .iter()
                    .enumerate()
                    .map(|(i, _)| QueryMap::arg(i))
                    .collect::<Vec<_>>();

                let returning = returns
                    .map(|col| format!("RETURNING {}", col))
                    .unwrap_or_default();

                format!(
                    "INSERT INTO {} ({}) VALUES ({}) {}",
                    table_name,
                    columns.join(", "),
                    values.join(", "),
                    returning
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
                    .map(|(i, (col, _))| format!("{col} = {}", QueryMap::arg(i)))
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

    #[cfg(feature = "serde")]
    use serde_json::json;

    #[derive(Model, FromRow)]
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

        let mut qb = QB::new(&pool).with_modifiers(&modifiers);
        qb.select::<TestUserModel>().await.ok();

        assert_eq!(
            qb.sql_str(),
            "SELECT * FROM users WHERE id = $1 AND business_id = $2 OR pid = $3 LIMIT 1"
                .to_string()
        );

        qb.select_all::<TestUserModel>().await.ok();
    }

    #[tokio::test]
    async fn test_update_query_sql_str() {
        let pool = pool().await;
        let modifiers = QueryModifiers::new()
            .with_filter(("id", 32))
            .and(eq("business_id", 32))
            .or(eq("pid", Uuid::new_v4()));

        #[cfg(not(feature = "serde"))]
        let map = query_map! { "users",
          "name": "Demo User",
          "age": 34
        };

        #[cfg(feature = "serde")]
        let map = &json! ({
          "name": "Demo User",
          "age": 34
        });

        let mut qb = QB::new(&pool);
        qb.set_modifiers(&modifiers);

        #[cfg(feature = "serde")]
        qb.update::<TestUserModel, _>(map).await.ok();

        #[cfg(not(feature = "serde"))]
        qb.update::<TestUserModel>(map).await.ok();

        assert_eq!(
            qb.sql_str(),
            "UPDATE users SET age = $1, name = $2 WHERE id = $3 AND business_id = $4 OR pid = $5"
        );
    }

    // #[tokio::test]
    // async fn test_insert_query_sql_str() {
    //     let pool = pool().await;
    //
    //     #[cfg(not(feature = "serde"))]
    //     let map = query_map! {
    //       "name": "Demo User",
    //       "age": 34
    //     };
    //
    //     #[cfg(feature = "serde")]
    //     let map = &json! ({
    //       "name": "Demo User",
    //       "age": 34
    //     });
    //
    //     let mut qb = QB::new(&pool);
    //     qb.insert::<TestUserModel, _>(map).await.ok();
    //
    //     assert_eq!(
    //         qb.sql_str(),
    //         "INSERT INTO users (age, name) VALUES ($1, $2)"
    //     )
    // }

    #[tokio::test]
    async fn test_order_by() {
        let pool = pool().await;
        let modifiers =
            QueryModifiers::new().with_sort(query_sort!(QuerySortDir::DESC, "created_at"));

        let mut qb = QB::new(&pool).with_modifiers(&modifiers);
        qb.select::<TestUserModel>().await.ok();

        assert_eq!(qb.sql_str(), "SELECT * FROM users ORDER BY created_at DESC");
    }
}
