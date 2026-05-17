use crate::{DbPool, QbEngine, QB};
use sqlx::postgres::PgRow;
use sqlx::{Database, Decode, Encode, FromRow, Type};
use std::future::Future;

pub trait Model: Sized + Send + Unpin + for<'r> FromRow<'r, PgRow> {
    const TABLE_NAME: &'static str;

    fn fetch<'q>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<Self, sqlx::Error>> {
        async { qb.query_fetch_one(db_pool).await }
    }

    fn fetch_all<'q>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<Vec<Self>, sqlx::Error>> {
        async { qb.query_fetch_all(db_pool).await }
    }

    fn fetch_scalar<'q, R>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<R, sqlx::Error>>
    where
        R: Send + Unpin,
        R: for<'r> Encode<'r, QbEngine> + for<'r> Decode<'r, QbEngine> + Type<QbEngine>,
        (R,): for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        async { qb.query_fetch_scalar(db_pool).await }
    }

    fn fetch_scalar_all<'q, R>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<Vec<R>, sqlx::Error>>
    where
        R: Send + Unpin,
        R: for<'r> Encode<'r, QbEngine> + for<'r> Decode<'r, QbEngine> + Type<QbEngine>,
        (R,): for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        async { qb.query_fetch_scalar_all(db_pool).await }
    }

    fn fetch_fields<'q, R>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<R, sqlx::Error>>
    where
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        async { qb.query_fetch_fields(db_pool).await }
    }

    fn fetch_fields_all<'q, R>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<Vec<R>, sqlx::Error>>
    where
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        async { qb.query_fetch_fields_all(db_pool).await }
    }
}

pub trait ModelInsertArg<M: Model> {
    type Returns;

    fn insert(
        self,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<Self::Returns, sqlx::Error>> + Send;
}
