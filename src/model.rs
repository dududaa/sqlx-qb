use crate::apis::{delete_query, select_fields_query, select_query, update_query};
use crate::extension::QueryExt;
use crate::DbPool;
use crate::QuerySet;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use std::future::Future;

pub trait Model: Sized + Send + Unpin + for<'r> FromRow<'r, PgRow> {
    const TABLE_NAME: &'static str;

    fn fetch_one(
        db_pool: &DbPool,
        filters: QueryExt,
    ) -> impl Future<Output = Result<Self, sqlx::Error>> + Send {
        async {
            let query = select_query(Self::TABLE_NAME, filters.with_limit(1));
            query.fetch_one(db_pool).await
        }
    }

    fn fetch_all(
        db_pool: &DbPool,
        filters: QueryExt,
    ) -> impl Future<Output = Result<Vec<Self>, sqlx::Error>> {
        async {
            let query = select_query(Self::TABLE_NAME, filters);
            query.fetch_all(db_pool).await
        }
    }

    /// Sometimes, all you want is a selected list of fields, not the complete model.
    fn fetch_scalar_one<'f, R>(
        db_pool: &DbPool,
        filters: QueryExt<'f>,
        fields: Vec<&str>,
    ) -> impl Future<Output = Result<R, sqlx::Error>> + Send
    where
        R: Send + Unpin,
        (R,): for<'r> FromRow<'r, PgRow>,
    {
        async {
            let query = select_fields_query(Self::TABLE_NAME, fields, filters.with_limit(1));
            query.fetch_scalar_one(db_pool).await
        }
    }

    fn fetch_fields_all<'f, R>(
        db_pool: &DbPool,
        filters: QueryExt<'f>,
        fields: Vec<&str>,
    ) -> impl Future<Output = Result<Vec<R>, sqlx::Error>> + Send
    where
        R: Send + Unpin + for<'r> FromRow<'r, PgRow>,
    {
        async {
            let query = select_fields_query(Self::TABLE_NAME, fields, filters);
            query.fetch_fields_all(db_pool).await
        }
    }

    fn update<'q>(
        db_pool: &DbPool,
        set: QuerySet<'q>,
        filters: QueryExt<'q>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send {
        async {
            let query = update_query(Self::TABLE_NAME, set, filters);
            query.execute(db_pool).await?;

            Ok(())
        }
    }

    fn delete(
        db_pool: &DbPool,
        filters: QueryExt,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send {
        async {
            let query = delete_query(Self::TABLE_NAME, filters);
            query.execute(db_pool).await?;

            Ok(())
        }
    }
}

pub trait ModelInsertArg<M: Model> {
    type Returns;

    fn insert(self, db_pool: &DbPool) -> impl Future<Output = anyhow::Result<Self::Returns>>;
}
