use sqlx::query::{Query as SqlxQuery, QueryAs as SqlxQueryAs, QueryScalar as SqlxQueryScalar};
use sqlx::{AssertSqlSafe, Database, Decode, Encode, FromRow, Type};

pub struct Query<'q, DB: Database>(SqlxQuery<'q, DB, DB::Arguments>);

impl<'q, DB: Database> Query<'q, DB> {
    pub fn new(sql: &str) -> Self {
        Self(sqlx::query(AssertSqlSafe(sql)))
    }
}

impl<'q, DB: Database> QueryWrapper<'q, DB> for Query<'q, DB> {
    type Inner = SqlxQuery<'q, DB, DB::Arguments>;
    fn bind<T: 'q + Encode<'q, DB> + Type<DB>>(self, value: T) -> Self {
        Self(self.0.bind(value))
    }

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

pub struct QueryAs<'q, DB: Database, M>(SqlxQueryAs<'q, DB, M, DB::Arguments>);

impl<'q, M, DB: Database> QueryAs<'q, DB, M>
where
    M: Sized + Send + Unpin + for<'r> FromRow<'r, DB::Row>,
{
    pub fn new(sql: &str) -> Self {
        Self(sqlx::query_as(AssertSqlSafe(sql)))
    }
}

impl<'q, M, DB: Database> QueryWrapper<'q, DB> for QueryAs<'q, DB, M>
where
    M: Sized + Send + Unpin + for<'r> FromRow<'r, DB::Row>,
{
    type Inner = SqlxQueryAs<'q, DB, M, DB::Arguments>;

    fn bind<T: 'q + Encode<'q, DB> + Type<DB>>(self, value: T) -> Self {
        Self(self.0.bind(value))
    }

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

pub struct QueryScalar<'q, DB: Database, R>(SqlxQueryScalar<'q, DB, R, DB::Arguments>);

impl<'q, R, DB: Database> QueryScalar<'q, DB, R>
where
    R: 'q + Encode<'q, DB> + Decode<'q, DB> + Type<DB>,
    (R,): for<'r> FromRow<'r, DB::Row>,
{
    pub fn new(sql: &str) -> Self {
        Self(sqlx::query_scalar(AssertSqlSafe(sql)))
    }
}

impl<'q, R, DB: Database> QueryWrapper<'q, DB> for QueryScalar<'q, DB, R> {
    type Inner = SqlxQueryScalar<'q, DB, R, DB::Arguments>;
    fn bind<T: 'q + Encode<'q, DB> + Type<DB>>(self, value: T) -> Self {
        Self(self.0.bind(value))
    }

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

pub trait QueryWrapper<'q, DB: Database> {
    type Inner;
    fn bind<T: 'q + Encode<'q, DB> + Type<DB>>(self, value: T) -> Self;

    fn into_inner(self) -> Self::Inner;
}
