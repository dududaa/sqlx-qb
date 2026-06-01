use crate::QB;
use sqlx::{Database, Executor, IntoArguments};
use std::future::Future;

pub trait Model: Sized + Send + Unpin {
    const TABLE_NAME: &'static str;
    const PRIMARY_COLUMN: &'static str;
    type InsertReturns;

    fn insert<'q, DB, E>(
        qb: &mut QB<'q, DB, E>,
    ) -> impl Future<Output = Result<Self::InsertReturns, sqlx::Error>>
    where
        DB: Database,
        E: Executor<'q, Database = DB> + Clone,
        DB::Arguments: IntoArguments<DB>,
        String: sqlx::Encode<'q, DB>,
        String: sqlx::Type<DB>;
}

pub trait ModelInsertArg<M: Model> {
    type Returns;

    fn insert<DB, P>(
        self,
        pool: &P,
    ) -> impl Future<Output = Result<Self::Returns, sqlx::Error>> + Send;
}
