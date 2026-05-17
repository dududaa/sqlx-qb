use crate::{DbPool, QbEngine, QbResult, QB};
use sqlx::{Database, FromRow};
use std::future::Future;

pub trait Model: Sized + Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row> {
    const TABLE_NAME: &'static str;

    fn insert<'q>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<QbResult, sqlx::Error>> {
        async { qb.execute(db_pool).await }
    }

    fn select<'q>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<Self, sqlx::Error>> {
        async { qb.fetch_one(db_pool).await }
    }

    fn select_all<'q>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<Vec<Self>, sqlx::Error>> {
        async { qb.fetch_all(db_pool).await }
    }

    fn select_fields<'q, R>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<R, sqlx::Error>>
    where
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        async { qb.fetch_fields_one(db_pool).await }
    }

    fn select_fields_all<'q, R>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<Vec<R>, sqlx::Error>>
    where
        R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    {
        async { qb.fetch_fields_all(db_pool).await }
    }

    fn update<'q>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<QbResult, sqlx::Error>> {
        async { qb.execute(db_pool).await }
    }

    fn delete<'q>(
        qb: &'q QB<'q, Self>,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<QbResult, sqlx::Error>> {
        async { qb.execute(db_pool).await }
    }
}

pub trait ModelInsertArg<M: Model> {
    type Returns;

    fn insert(
        self,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<Self::Returns, sqlx::Error>> + Send;
}
