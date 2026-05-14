use crate::apis::{delete_query, select_fields_query, select_query, update_query};
use crate::extension::QueryExt;
use crate::QuerySet;
use crate::DbPool;
use sqlx::postgres::PgRow;
use sqlx::FromRow;

pub trait Model: Sized + Send + Unpin + for<'r> FromRow<'r, PgRow> {
    const TABLE_NAME: &'static str;

    fn fetch_one<'f>(db_pool: &DbPool, filters: QueryExt<'f>) -> Result<Self, sqlx::Error> {
        let query = select_query(&Self::TABLE_NAME, filters.with_limit(1));
        query.fetch_one(db_pool).await
    }

    async fn fetch_all<'f>(
        db_pool: &DbPool,
        filters: QueryExt<'f>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let query = select_query(&Self::TABLE_NAME, filters);
        query.fetch_all(db_pool).await
    }

    /// Sometimes, all you want is a selected list of fields, not the complete model.
    async fn fetch_scalar_one<'f, R>(
        db_pool: &DbPool,
        filters: QueryExt<'f>,
        fields: Vec<&str>,
    ) -> Result<R, sqlx::Error>
    where
        R: Send + Unpin,
        (R,): for<'r> FromRow<'r, PgRow>,
    {
        let query = select_fields_query(&Self::TABLE_NAME, fields, filters.with_limit(1));
        query.fetch_scalar_one(db_pool).await
    }

    async fn fetch_fields_all<'f, R>(
        db_pool: &DbPool,
        filters: QueryExt<'f>,
        fields: Vec<&str>,
    ) -> Result<Vec<R>, sqlx::Error>
    where
        R: Send + Unpin + for<'r> FromRow<'r, PgRow>,
    {
        let query = select_fields_query(&Self::TABLE_NAME, fields, filters);
        query.fetch_fields_all(db_pool).await
    }

    async fn update<'q>(
        db_pool: &DbPool,
        set: QuerySet<'q>,
        filters: QueryExt<'q>,
    ) -> anyhow::Result<()> {
        let query = update_query(&Self::TABLE_NAME, set, filters);
        query.execute(db_pool).await?;

        Ok(())
    }

    async fn delete<'f>(db_pool: &DbPool, filters: QueryExt<'f>) -> anyhow::Result<()> {
        let query = delete_query(&Self::TABLE_NAME, filters);
        query.execute(db_pool).await?;

        Ok(())
    }
}

pub trait ModelInsertArg<M: Model> {
    type Returns;

    async fn insert(self, db_pool: &DbPool) -> anyhow::Result<Self::Returns>;
}
