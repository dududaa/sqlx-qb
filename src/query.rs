use sqlx::query::{Query as SqlxQuery, QueryAs as SqlxQueryAs, QueryScalar as SqlxQueryScalar};
use sqlx::{
    Arguments, AssertSqlSafe, Database, Decode, Encode, Error, Executor, FromRow, IntoArguments,
    Type,
};
use std::marker::PhantomData;

pub trait QueryWrapper<'q, DB: Database>
where
    String: sqlx::Type<DB>,
    String: sqlx::Encode<'q, DB>,
{
    fn values(&self) -> Vec<String>;

    fn bind_values(mut self) -> Self
    where
        Self: Sized,
    {
        for value in self.values() {
            self = self.bind(value.to_string())
        }

        self
    }

    fn bind<T>(self, value: T) -> Self
    where
        T: sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + 'q;
}

macro_rules! impl_wrapper {
    ( $instance:ident ) => {
        pub struct $instance<DB, A>
        where
            DB: Database,
            A: IntoArguments<DB>,
        {
            sql: String,
            args: A,
            values: Vec<String>,
            _db: PhantomData<DB>,
        }

        impl<DB, A> $instance<DB, A>
        where
            DB: Database,
            A: IntoArguments<DB> + Default,
        {
            pub fn new(sql: String, values: Vec<String>) -> Self {
                Self {
                    sql,
                    values,
                    args: Default::default(),
                    _db: PhantomData,
                }
            }
        }

        impl<'q, DB: Database> QueryWrapper<'q, DB> for $instance<DB, DB::Arguments>
        where
            DB: Database,
            for<'a> DB::Arguments: IntoArguments<DB>,
            String: sqlx::Type<DB>,
            String: sqlx::Encode<'q, DB>,
        {
            fn values(&self) -> Vec<String> {
                self.values.clone()
            }

            fn bind<T>(mut self, value: T) -> Self
            where
                T: sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + 'q,
            {
                let _ = self.args.add(value);
                self
            }
        }
    };
}

impl_wrapper!(Query);
impl_wrapper!(QueryAs);
impl_wrapper!(QueryScalar);

impl<'q, DB> Query<DB, DB::Arguments>
where
    DB: Database,
    for<'a> DB::Arguments: IntoArguments<DB>,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
{
    fn build(mut self) -> SqlxQuery<'q, DB, DB::Arguments> {
        self = self.bind_values();
        sqlx::query_with(AssertSqlSafe(self.sql), self.args)
    }

    pub(crate) async fn execute<E: Executor<'q, Database = DB>>(
        self,
        executor: E,
    ) -> Result<(), Error> {
        self.build().execute(executor).await?;
        Ok(())
    }
}

impl<'q, DB> QueryAs<DB, DB::Arguments>
where
    DB: Database,
    for<'a> DB::Arguments: IntoArguments<DB>,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
{
    fn build<M>(mut self) -> SqlxQueryAs<'q, DB, M, DB::Arguments>
    where
        M: Sized + Send + Unpin + for<'r> FromRow<'r, DB::Row>,
    {
        self = self.bind_values();
        sqlx::query_as_with(AssertSqlSafe(self.sql), self.args)
    }

    pub(crate) async fn fetch_one<R, E>(self, executor: E) -> Result<R, Error>
    where
        E: Executor<'q, Database = DB>,
        R: Send + Unpin + for<'r> FromRow<'r, DB::Row>,
    {
        self.build().fetch_one(executor).await
    }

    pub(crate) async fn fetch_all<R, E>(self, executor: E) -> Result<Vec<R>, Error>
    where
        E: Executor<'q, Database = DB>,
        R: Send + Unpin + for<'r> FromRow<'r, DB::Row>,
    {
        self.build().fetch_all(executor).await
    }
}

impl<'q, DB> QueryScalar<DB, DB::Arguments>
where
    DB: Database,
    for<'a> DB::Arguments: IntoArguments<DB>,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
{
    fn build<R>(mut self) -> SqlxQueryScalar<'q, DB, R, DB::Arguments>
    where
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        self = self.bind_values();
        sqlx::query_scalar_with(AssertSqlSafe(self.sql), self.args)
    }

    pub(crate) async fn fetch_one<R, E>(self, executor: E) -> Result<R, Error>
    where
        E: Executor<'q, Database = DB>,
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        self.build().fetch_one(executor).await
    }

    pub(crate) async fn fetch_all<R, E>(self, executor: E) -> Result<Vec<R>, Error>
    where
        E: Executor<'q, Database = DB>,
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        self.build().fetch_all(executor).await
    }
}
