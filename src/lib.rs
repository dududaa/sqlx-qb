mod extensions;
mod map;
mod model;
mod modifiers;
mod query;
mod types;
// mod value;

pub mod prelude {
    pub use crate::extensions::*;
    pub use crate::map::*;
    pub use crate::model::*;
    pub use crate::modifiers::*;
    pub use crate::query_map;
    pub use crate::query_sort;
    pub use crate::DbPool;
    pub use crate::QB;
    pub use qb_macro::QbModel;
    pub use sqlx::{FromRow, Database, Executor, IntoArguments};
    pub use std::future::Future;
}

use types::*;

use crate::model::{Model, ModelInsertArg};
use crate::query::{Query, QueryAs, QueryScalar};
use map::QueryMap;
use modifiers::QueryModifiers;
use sqlx::{Database, Decode, Encode, Error, Executor, FromRow, IntoArguments, Pool, Type};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

pub type DbPool = Pool<QbEngine>;

pub struct QB<'q, DB, P>
where
    DB: Database,
    P: Executor<'q, Database = DB> + Clone,
{
    inner: SqlxQb<'q, DB, P>,
}

impl<'q, DB, P> QB<'q, DB, P>
where
    DB: Database,
    P: Executor<'q, Database = DB> + Clone,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
    DB::Arguments: IntoArguments<DB>,
{
    pub fn new(pool: P) -> Self {
        Self {
            inner: SqlxQb::new(pool),
        }
    }

    pub fn pool(&'q self) -> &'q P {
        &self.pool
    }

    pub fn sql_str(&self) -> String {
        self.inner.sql_str()
    }

    pub async fn insert<M: Model>(
        &'q mut self,
        map: QueryMap<'q>,
    ) -> Result<M::InsertReturns, Error> {
        self.with_command(QueryCommand::Insert(M::TABLE_NAME, map));
        let modifiers = self.modifiers;
        self.reset_modifiers();

        let result = M::insert(self).await;
        if let Some(modifiers) = modifiers {
            self.set_modifiers(modifiers);
        }

        result
    }

    pub async fn insert_args<M, A>(&self, args: A) -> Result<A::Returns, Error>
    where
        M: Model,
        A: ModelInsertArg<M>,
    {
        args.insert::<DB, P>(&self.pool).await
    }

    pub async fn select<M>(&mut self) -> Result<M, Error>
    where
        M: Model + for<'r> sqlx::FromRow<'r, DB::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::WildCard,
            M::TABLE_NAME,
        ));

        self.fetch_one().await
    }

    pub async fn select_all<M: Model + for<'r> sqlx::FromRow<'r, DB::Row>>(
        &mut self,
    ) -> Result<Vec<M>, Error>
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::WildCard,
            M::TABLE_NAME,
        ));

        self.fetch_all().await
    }

    pub async fn select_fields<M>(
        &mut self,
        fields: impl Into<Vec<&'q str>>,
    ) -> Result<DB::Row, Error>
    where
        M: Model,
        for<'r> <DB as Database>::Row: FromRow<'r, <DB as Database>::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::Fields(fields.into()),
            M::TABLE_NAME,
        ));

        self.fetch_fields_one().await
    }

    pub async fn select_fields_all<M>(
        &mut self,
        fields: impl Into<Vec<&'q str>>,
    ) -> Result<Vec<DB::Row>, Error>
    where
        M: Model,
        for<'r> <DB as Database>::Row: FromRow<'r, <DB as Database>::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::Fields(fields.into()),
            M::TABLE_NAME,
        ));

        self.fetch_fields_all().await
    }

    pub async fn select_scalar<M, R>(&mut self, field: &'q str) -> Result<R, Error>
    where
        M: Model,
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::Fields([field].into()),
            M::TABLE_NAME,
        ));

        self.fetch_scalar_one().await
    }

    pub async fn select_scalar_all<M, R>(&mut self, field: &'q str) -> Result<Vec<R>, Error>
    where
        M: Model,
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        self.with_command(QueryCommand::Select(
            QuerySelectCommand::Fields([field].into()),
            M::TABLE_NAME,
        ));

        self.fetch_scalar_all().await
    }

    pub async fn update<M: Model>(&mut self, set: QueryMap<'q>) -> Result<(), Error>
    {
        self.with_command(QueryCommand::Update(M::TABLE_NAME, set));
        self.execute().await
    }

    pub async fn delete<M: Model>(mut self) -> Result<(), Error>
    {
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

impl<'q, DB, P> SqlxQb<'q, DB, P>
where
    DB: Database,
    P: Executor<'q, Database = DB> + Clone,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
{
    fn new(pool: P) -> Self {
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

        if let QueryCommand::Insert(_, map) = &self.cmd {
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

    pub(crate) async fn fetch_one<M: Model + for<'r> sqlx::FromRow<'r, DB::Row>>(
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
        M: Model + for<'r> sqlx::FromRow<'r, DB::Row>,
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

    pub(crate) async fn fetch_fields_one(&self) -> Result<DB::Row, Error>
    where
        DB::Arguments: IntoArguments<DB>,
        for<'r> <DB as Database>::Row: FromRow<'r, <DB as Database>::Row>,
    {
        let sql = self.sql_str();
        let args = self.args.clone();
        let query = QueryAs::new(sql, args);

        query.fetch_one(self.pool.clone()).await
    }

    pub(crate) async fn fetch_fields_all(&self) -> Result<Vec<DB::Row>, Error>
    where
        DB::Arguments: IntoArguments<DB>,
        for<'r> <DB as Database>::Row: FromRow<'r, <DB as Database>::Row>,
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
                    .keys()
                    .map(|col| col.to_string())
                    .collect::<Vec<_>>();

                let values = map
                    .inner()
                    .iter()
                    .enumerate()
                    .map(|(i, _)| QueryMap::arg(i))
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
    use sqlx::any::AnyPoolOptions;
    use sqlx::{AnyPool, FromRow};
    use uuid::Uuid;

    #[derive(QbModel, FromRow)]
    #[model(table_name = "users")]
    struct TestUserModel {}

    async fn pool() -> AnyPool {
        // let connection_options = SqliteConnectOptions::from_str("file::memory:?cache=shared")
        //     .unwrap()
        //     .create_if_missing(true);
        //
        // SqlitePoolOptions::new()
        //     .max_connections(1)
        //     .connect_with(connection_options)
        //     .await
        //     .unwrap()

        #[cfg(feature = "any")]
        AnyPoolOptions::new()
            .max_connections(5)
            .connect("test")
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

        let mut qb = QB::new(&pool);
        qb.set_modifiers(&modifiers);
        qb.update::<TestUserModel>(set).await.ok();

        assert_eq!(
            qb.sql_str(),
            "UPDATE users SET age = $1, name = $2 WHERE id = $3 AND business_id = $4 OR pid = $5"
        );
    }

    // #[tokio::test]
    // async fn test_insert_query_sql_str() {
    //     let pool = pool().await;
    //     let map = query_map! {
    //       "name": "Demo User",
    //       "age": 34
    //     };
    //
    //     let mut qb = QB::new(&pool);
    //     qb.insert::<TestUserModel>(map).await.ok();
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
