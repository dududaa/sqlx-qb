use crate::QB;
use sqlx::{Database, Decode, Encode, Executor, IntoArguments, Type};
use std::future::Future;
use crate::prelude::FromRow;

pub trait Model: Sized + Send + Unpin {
    const TABLE_NAME: &'static str;
    const PRIMARY_COLUMN: &'static str;
    type InsertReturns;

    fn insert<'q, DB, E>(qb: &mut QB<'q, DB, E>) -> impl Future<Output = Result<(), sqlx::Error>>
    where
        DB: Database,
        E: Executor<'q, Database = DB> + Clone,
        DB::Arguments: IntoArguments<DB>,
        String: sqlx::Encode<'q, DB>,
        String: sqlx::Type<DB> {
        async {
            let _ = qb.execute().await?;
            Ok(())
        }
    }

    fn insert_returns<'q, DB, E>(
        qb: &mut QB<'q, DB, E>,
    ) -> impl Future<Output = Result<Self::InsertReturns, sqlx::Error>>
    where
        DB: Database,
        E: Executor<'q, Database = DB> + Clone,
        DB::Arguments: IntoArguments<DB>,
        String: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        Self::InsertReturns: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB> + Send + Unpin + 'q,
        (Self::InsertReturns, ): for<'r> FromRow<'r, DB::Row>
    {
        async {
            let res = qb.fetch_scalar_one().await?;
            Ok(res)
        }
    }
}

pub trait ModelInsertArg<M, E>
where
    M: Model,
{
    type Returns;

    fn insert(self, pool: E) -> impl Future<Output = Result<Self::Returns, sqlx::Error>> + Send;
}
