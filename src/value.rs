use crate::query::QueryWrapper;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) enum QbValue<'q> {
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
    pub(super) fn bind<Q: QueryWrapper<'q>>(self, query: Q) -> Q {
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
