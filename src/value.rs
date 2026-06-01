use crate::query::QueryWrapper;

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};
use sqlx::Database;
#[cfg(feature = "uuid")]
use uuid::Uuid;

#[derive(Clone)]
pub enum QbValue<'q> {
    Int(i32),
    SmallInt(i16),
    BigInt(i64),
    Text(&'q str),
    #[cfg(feature = "uuid")]
    Uuid(Uuid),
    #[cfg(feature = "chrono")]
    DateTime(DateTime<Utc>),
}

//
impl<'q> QbValue<'q> {
    pub(super) fn bind<DB: Database, Q: QueryWrapper<'q, DB>>(self, query: Q) -> Q {
        match self {
            QbValue::SmallInt(i) => query.bind(i),
            QbValue::Int(i) => query.bind(i),
            QbValue::BigInt(b) => query.bind(b),
            QbValue::Text(s) => query.bind(s),

            #[cfg(feature = "uuid")]
            QbValue::Uuid(u) => query.bind(u),

            #[cfg(feature = "uuid")]
            QbValue::DateTime(d) => query.bind(d),
        }
    }

    pub(crate) fn arg(idx: usize) -> String {
        if !cfg!(feature = "mysql") {
            format!("${}", idx + 1)
        } else {
            "?".to_string()
        }
    }
}

#[cfg(feature = "uuid")]
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

#[cfg(feature = "chrono")]
impl<'q> From<DateTime<Utc>> for QbValue<'q> {
    fn from(dt: DateTime<Utc>) -> Self {
        QbValue::DateTime(dt)
    }
}
