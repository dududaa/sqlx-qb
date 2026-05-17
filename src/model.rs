use crate::DbPool;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use std::future::Future;

pub trait Model: Sized + Send + Unpin + for<'r> FromRow<'r, PgRow> {
    const TABLE_NAME: &'static str;
}

pub trait ModelInsertArg<M: Model> {
    type Returns;

    fn insert(
        self,
        db_pool: &DbPool,
    ) -> impl Future<Output = Result<Self::Returns, sqlx::Error>> + Send;
}
