use crate::QbEngine;
use sqlx::query::{Query as SqlxQuery, QueryAs as SqlxQueryAs, QueryScalar as SqlxQueryScalar};
use sqlx::{Database, Decode, Encode, FromRow, Type};

pub struct Query<'q, DB: Database>(SqlxQuery<'q, DB, DB::Arguments<'q>>);

impl<'q> Query<'q, QbEngine> {
    pub fn new(sql: &'q str) -> Self {
        Self(sqlx::query(sql))
    }
}

impl<'q> QueryWrapper<'q> for Query<'q, QbEngine> {
    type Inner = SqlxQuery<'q, QbEngine, <QbEngine as Database>::Arguments<'q>>;
    fn bind<T: 'q + Encode<'q, QbEngine> + Type<QbEngine>>(self, value: T) -> Self {
        Self(self.0.bind(value))
    }

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

pub struct QueryAs<'q, DB: Database, M>(SqlxQueryAs<'q, DB, M, DB::Arguments<'q>>);

impl<'q, M> QueryAs<'q, QbEngine, M>
where
    M: Sized + Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
{
    pub fn new(sql: &'q str) -> Self {
        Self(sqlx::query_as(sql))
    }
}

impl<'q, M> QueryWrapper<'q> for QueryAs<'q, QbEngine, M>
where
    M: Sized + Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
{
    type Inner = SqlxQueryAs<'q, QbEngine, M, <QbEngine as Database>::Arguments<'q>>;

    fn bind<T: 'q + Encode<'q, QbEngine> + Type<QbEngine>>(self, value: T) -> Self {
        Self(self.0.bind(value))
    }

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}
pub struct QueryScalar<'q, DB: Database, R>(SqlxQueryScalar<'q, DB, R, DB::Arguments<'q>>);

impl<'q, R> QueryScalar<'q, QbEngine, R>
where
    R: 'q + Encode<'q, QbEngine> + Decode<'q, QbEngine> + Type<QbEngine>,
    (R,): for<'r> FromRow<'r, <QbEngine as Database>::Row>,
{
    pub fn new(sql: &'q str) -> Self {
        Self(sqlx::query_scalar(sql))
    }
}

impl<'q, R> QueryWrapper<'q> for QueryScalar<'q, QbEngine, R> {
    type Inner = SqlxQueryScalar<'q, QbEngine, R, <QbEngine as Database>::Arguments<'q>>;
    fn bind<T: 'q + Encode<'q, QbEngine> + Type<QbEngine>>(self, value: T) -> Self {
        Self(self.0.bind(value))
    }

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

pub trait QueryWrapper<'q> {
    type Inner;
    fn bind<T: 'q + Encode<'q, QbEngine> + Type<QbEngine>>(self, value: T) -> Self;

    fn into_inner(self) -> Self::Inner;
}
