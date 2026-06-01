use crate::query::{QueryAs, QueryWrapper};
use crate::value::QbValue;
use crate::QB;
use sqlx::{Database, Executor, FromRow, IntoArguments};
use std::future::Future;

pub trait Model: Sized + Send + Unpin {
    const TABLE_NAME: &'static str;
    const PRIMARY_COLUMN: &'static str;
    type InsertReturns;

    fn insert<'q, DB, E>(
        qb: &'q QB<'q, DB, E>,
    ) -> impl Future<Output = Result<Self::InsertReturns, sqlx::Error>>
    where
        DB: Database,
        E: Executor<'q, Database = DB> + Clone;
    //
    fn get<'q, DB, E>(
        qb: &'q QB<'q, DB, E>,
        value: QbValue<'q>,
    ) -> impl Future<Output = Result<Self, sqlx::Error>>
    where
        DB: Database,
        E: Executor<'q, Database = DB> + Clone,
        for<'r> Self: FromRow<'r, <DB as Database>::Row>,
        <DB as Database>::Arguments: IntoArguments<DB>,
    {
        async {
            let arg = QbValue::arg(0);
            let sql = format!(
                "SELECT * FROM {} WHERE {} = {} LIMIT 1",
                Self::TABLE_NAME,
                Self::PRIMARY_COLUMN,
                arg
            );
            let query = QueryAs::new(&sql);

            let model = value
                .bind(query)
                .into_inner()
                .fetch_one(qb.pool().clone())
                .await?;
            Ok(model)
        }
    }
    //
    // fn select<'q>(qb: &'q QB<'q, Self>) -> impl Future<Output = Result<Self, sqlx::Error>> {
    //     async { qb.fetch_one(qb.pool()).await }
    // }
    //
    // fn select_all<'q>(
    //     qb: &'q QB<'q, Self>,
    // ) -> impl Future<Output = Result<Vec<Self>, sqlx::Error>> {
    //     async { qb.fetch_all(qb.pool()).await }
    // }
    //
    // fn select_fields<'q, R>(qb: &'q QB<'q, Self>) -> impl Future<Output = Result<R, sqlx::Error>>
    // where
    //     R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    // {
    //     async { qb.fetch_fields_one(qb.pool()).await }
    // }
    //
    // fn select_fields_all<'q, R>(
    //     qb: &'q QB<'q, Self>,
    // ) -> impl Future<Output = Result<Vec<R>, sqlx::Error>>
    // where
    //     R: Send + Unpin + for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    // {
    //     async { qb.fetch_fields_all(qb.pool()).await }
    // }
    //
    // fn select_scalar<'q, R>(qb: &'q QB<'q, Self>) -> impl Future<Output = Result<R, sqlx::Error>>
    // where
    //     R: Send + Unpin,
    //     R: for<'r> Encode<'r, QbEngine> + for<'r> Decode<'r, QbEngine> + Type<QbEngine>,
    //     (R,): for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    // {
    //     async { qb.fetch_scalar_one(qb.pool()).await }
    // }
    //
    // fn select_scalar_all<'q, R>(
    //     qb: &'q QB<'q, Self>,
    // ) -> impl Future<Output = Result<Vec<R>, sqlx::Error>>
    // where
    //     R: Send + Unpin,
    //     R: for<'r> Encode<'r, QbEngine> + for<'r> Decode<'r, QbEngine> + Type<QbEngine>,
    //     (R,): for<'r> FromRow<'r, <QbEngine as Database>::Row>,
    // {
    //     async { qb.fetch_scalar_all(qb.pool()).await }
    // }
    //
    // fn update<'q>(qb: &'q QB<'q, Self>) -> impl Future<Output = Result<QbResult, sqlx::Error>> {
    //     async { qb.execute(qb.pool()).await }
    // }
    //
    // fn delete<'q>(qb: &'q QB<'q, Self>) -> impl Future<Output = Result<QbResult, sqlx::Error>> {
    //     async { qb.execute(qb.pool()).await }
    // }
}

pub trait ModelInsertArg<M: Model> {
    type Returns;

    fn insert<DB, P>(
        self,
        pool: &P,
    ) -> impl Future<Output = Result<Self::Returns, sqlx::Error>> + Send;
}
