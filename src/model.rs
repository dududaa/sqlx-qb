use crate::map::QueryMap;
use crate::prelude::FromRow;
use crate::{QueryCommand, QB};
use sqlx::{Database, Decode, Encode, Error, Executor, IntoArguments, Type};
use std::future::Future;

pub trait Model<'q, DB, E>:
    ModelInsert<'q> + ModelSelect<'q, DB, E> + Sized + Send + Unpin
where
    DB: Database,
    E: Executor<'q, Database = DB> + Clone,
    DB::Arguments: IntoArguments<DB>,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
{
    const TABLE_NAME: &'static str;
    const PRIMARY_COLUMN: &'static str;
}

pub trait ModelInsert<'q> {
    type InsertReturns;

    fn insert<DB, E>(&'q self, qb: &mut QB<'q, DB, E>) -> impl Future<Output = Result<(), Error>>
    where
        DB: Database,
        E: Executor<'q, Database = DB> + Clone,
        DB::Arguments: IntoArguments<DB>,
        String: sqlx::Encode<'q, DB>,
        String: sqlx::Type<DB>,
    {
        async {
            self.execute_insert::<_, _, DB, E>(qb, None, async |qb: &QB<'q, DB, E>| {
                qb.execute().await
            })
            .await
        }
    }

    fn insert_returns<DB, E>(
        &'q self,
        qb: &mut QB<'q, DB, E>,
        returning: &'q str,
    ) -> impl Future<Output = Result<Self::InsertReturns, Error>>
    where
        DB: Database,
        E: Executor<'q, Database = DB> + Clone,
        DB::Arguments: IntoArguments<DB>,
        String: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        Self::InsertReturns:
            for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB> + Send + Unpin + 'q,
        (Self::InsertReturns,): for<'r> FromRow<'r, DB::Row>,
    {
        async {
            self.execute_insert::<_, _, DB, E>(qb, Some(returning), async |qb: &QB<'q, DB, E>| {
                qb.fetch_scalar_one().await
            })
            .await
        }
    }
    fn execute_insert<R, F, DB, E>(
        &'q self,
        qb: &mut QB<'q, DB, E>,
        returning: Option<&'q str>,
        execution: F,
    ) -> impl Future<Output = R>
    where
        F: AsyncFn(&QB<'q, DB, E>) -> R + 'q,
        DB: Database,
        E: Executor<'q, Database = DB> + Clone,
        DB::Arguments: IntoArguments<DB>,
        String: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        async move {
            qb.with_command(QueryCommand::Insert {
                table_name: self.table_name(),
                map: self.to_map(),
                returning,
            });

            let modifiers = qb.modifiers;

            qb.reset_modifiers();
            let res = execution(qb).await;

            if let Some(modifiers) = modifiers {
                qb.set_modifiers(modifiers);
            }

            res
        }
    }

    /// Return name of table
    fn table_name(&'q self) -> &'q str;

    fn to_map(&'q self) -> QueryMap;
}

pub trait ModelSelect<'q, DB, E>
where
    DB: Database,
    E: Executor<'q, Database = DB> + Clone,
    DB::Arguments: IntoArguments<DB>,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
{
    fn select<M>(qb: &mut QB<'q, DB, E>) -> impl Future<Output = Result<M, Error>>
    where
        M: Model<'q, DB, E> + for<'r> sqlx::FromRow<'r, DB::Row>,
    {
        async { qb.fetch_one().await }
    }

    fn select_all<M: Model<'q, DB, E> + for<'r> sqlx::FromRow<'r, DB::Row>>(
        qb: &mut QB<'q, DB, E>,
    ) -> impl Future<Output = Result<Vec<M>, Error>>
    where
        M: Model<'q, DB, E> + for<'r> sqlx::FromRow<'r, DB::Row>,
    {
        async { qb.fetch_all().await }
    }

    fn select_fields<M, R>(qb: &mut QB<'q, DB, E>) -> impl Future<Output = Result<R, Error>>
    where
        M: Model<'q, DB, E>,
        R: Send + Unpin + for<'r> FromRow<'r, DB::Row>,
    {
        async { qb.fetch_fields_one().await }
    }

    fn select_fields_all<M, R>(
        qb: &mut QB<'q, DB, E>,
    ) -> impl Future<Output = Result<Vec<R>, Error>>
    where
        M: Model<'q, DB, E>,
        R: Send + Unpin + for<'r> FromRow<'r, DB::Row>,
    {
        async { qb.fetch_fields_all().await }
    }

    fn select_scalar<M, R>(qb: &mut QB<'q, DB, E>) -> impl Future<Output = Result<R, Error>>
    where
        M: Model<'q, DB, E>,
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        async { qb.fetch_scalar_one().await }
    }

    fn select_scalar_all<M, R>(
        qb: &mut QB<'q, DB, E>,
    ) -> impl Future<Output = Result<Vec<R>, Error>>
    where
        M: Model<'q, DB, E>,
        R: Send + Unpin + 'q,
        R: for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB>,
        (R,): for<'r> FromRow<'r, DB::Row>,
    {
        async { qb.fetch_scalar_all().await }
    }
}
