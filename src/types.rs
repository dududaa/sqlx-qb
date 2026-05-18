
#[cfg(feature = "postgres")]
use sqlx::postgres::{PgQueryResult, Postgres};

#[cfg(feature = "mysql")]
use sqlx::mysql::{MySql, MySqlQueryResult};

#[cfg(feature = "sqlite")]
use sqlx::sqlite::{SqliteQueryResult, Sqlite};

#[cfg(feature = "any")]
use sqlx::any::{Any, AnyQueryResult};

#[cfg(feature = "postgres")]
pub(crate) type QbEngine = Postgres;

#[cfg(feature = "mysql")]
pub(crate) type QbEngine = MySql;

#[cfg(feature = "sqlite")]
pub(crate) type QbEngine = Sqlite;

#[cfg(feature = "any")]
pub(crate) type QbEngine = sqlx::Any;

#[cfg(feature = "postgres")]
pub(crate) type QbResult = PgQueryResult;

#[cfg(feature = "mysql")]
pub(crate) type QbResult = MySqlQueryResult;

#[cfg(feature = "sqlite")]
pub(crate) type QbResult = SqliteQueryResult;

#[cfg(feature = "any")]
pub(crate) type QbResult = AnyQueryResult;
