use crate::map::QueryMap;
use crate::prelude::FromRow;
use crate::{QueryCommand, QB};
use sqlx::{Database, Decode, Encode, Error, Executor, IntoArguments, Type};
use std::future::Future;

pub trait Model<'q, DB, E>: Sized + Send + Unpin
where
    DB: Database,
    E: Executor<'q, Database = DB> + Clone,
    DB::Arguments: IntoArguments<DB>,
    String: sqlx::Encode<'q, DB>,
    String: sqlx::Type<DB>,
{
    const TABLE_NAME: &'static str;
    const PRIMARY_COLUMN: &'static str;

    fn select(qb: &QB<'q, DB, E>) -> impl Future<Output = Result<Self, Error>>
    where
        Self: for<'r> sqlx::FromRow<'r, DB::Row>,
    {
        async { qb.fetch_one().await }
    }

    fn select_all(qb: &mut QB<'q, DB, E>) -> impl Future<Output = Result<Vec<Self>, Error>>
    where
        Self: for<'r> sqlx::FromRow<'r, DB::Row>,
    {
        async { qb.fetch_all().await }
    }
}

pub trait ModelInsert<'q, InsertReturns>: QueryMapInput<'q> {
    const TABLE_NAME: Option<&'q str> = None;

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
    ) -> impl Future<Output = Result<InsertReturns, Error>>
    where
        DB: Database,
        E: Executor<'q, Database = DB> + Clone,
        DB::Arguments: IntoArguments<DB>,
        String: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        InsertReturns:
            for<'r> Encode<'r, DB> + for<'r> Decode<'r, DB> + Type<DB> + Send + Unpin + 'q,
        (InsertReturns,): for<'r> FromRow<'r, DB::Row>,
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
            let table_name = self
                .table_name()
                .unwrap_or(qb.table_name().expect("missing table name"));

            qb.with_command(QueryCommand::Insert {
                table_name,
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
}

pub trait QueryMapInput<'q> {
    /// Return name of table
    fn table_name(&'q self) -> Option<&'q str>;

    fn to_map(&'q self) -> QueryMap;
}
