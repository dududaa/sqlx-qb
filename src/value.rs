use chrono::{DateTime, Utc};
use sqlx::postgres::{PgArguments, PgRow};
use sqlx::query::{Query, QueryAs, QueryScalar};
use sqlx::{FromRow, Postgres};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone)]
pub enum QbValue<'q> {
    Uuid(Uuid),
    Int(i32),
    SmallInt(i16),
    BigInt(i64),
    Text(&'q str),
    DateTime(DateTime<Utc>),
}

impl<'q> QbValue<'q> {
    pub(super) fn bind_to_query(
        self,
        query: Query<'q, Postgres, PgArguments>,
    ) -> Query<'q, Postgres, PgArguments> {
        match self {
            QbValue::SmallInt(i) => query.bind(i),
            QbValue::Int(i) => query.bind(i),
            QbValue::BigInt(b) => query.bind(b),
            QbValue::Uuid(u) => query.bind(u),
            QbValue::Text(s) => query.bind(s),
            QbValue::DateTime(d) => query.bind(d),
        }
    }

    pub(super) fn bind_to_query_as<M>(
        self,
        query: QueryAs<'q, Postgres, M, PgArguments>,
    ) -> QueryAs<'q, Postgres, M, PgArguments>
    where
        M: Sized + Send + Unpin + for<'r> FromRow<'r, PgRow>,
    {
        match self {
            QbValue::SmallInt(i) => query.bind(i),
            QbValue::Int(i) => query.bind(i),
            QbValue::BigInt(b) => query.bind(b),
            QbValue::Uuid(u) => query.bind(u),
            QbValue::Text(s) => query.bind(s),
            QbValue::DateTime(d) => query.bind(d),
        }
    }

    pub(super) fn bind_to_query_scalar<R>(
        self,
        query: QueryScalar<'q, Postgres, R, PgArguments>,
    ) -> QueryScalar<'q, Postgres, R, PgArguments> {
        match self {
            QbValue::SmallInt(i) => query.bind(i),
            QbValue::Int(i) => query.bind(i),
            QbValue::BigInt(b) => query.bind(b),
            QbValue::Uuid(u) => query.bind(u),
            QbValue::Text(s) => query.bind(s),
            QbValue::DateTime(d) => query.bind(d),
        }
    }
}

impl<'q> Display for QbValue<'q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QbValue::Uuid(v) => write!(f, "{}", v),
            QbValue::Int(v) => write!(f, "{}", v),
            QbValue::SmallInt(v) => write!(f, "{}", v),
            QbValue::BigInt(v) => write!(f, "{}", v),
            QbValue::Text(v) => write!(f, "{}", v),
            QbValue::DateTime(v) => write!(f, "{}", v),
        }
    }
}

impl<'q> From<Uuid> for QbValue<'q> {
    fn from(uuid: Uuid) -> Self {
        QbValue::Uuid(uuid)
    }
}

impl<'q> From<i32> for QbValue<'q> {
    fn from(int: i32) -> Self {
        QbValue::Int(int)
    }
}

impl<'q> From<i64> for QbValue<'q> {
    fn from(int: i64) -> Self {
        QbValue::BigInt(int)
    }
}

impl<'q> From<&'q str> for QbValue<'q> {
    fn from(text: &'q str) -> Self {
        QbValue::Text(text)
    }
}

impl<'q> From<i16> for QbValue<'q> {
    fn from(small_int: i16) -> Self {
        QbValue::SmallInt(small_int)
    }
}

impl<'q> From<DateTime<Utc>> for QbValue<'q> {
    fn from(dt: DateTime<Utc>) -> Self {
        QbValue::DateTime(dt)
    }
}
