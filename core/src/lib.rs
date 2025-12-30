pub mod crud;
pub mod database_info;
pub mod db_pool;
pub mod error;
pub mod executor;
pub mod macros_api;
pub mod query_builder;
pub mod traits;
pub mod transaction;
pub mod utils;

pub use database_info::DatabaseInfo;
pub use db_pool::{DbDriver, DbPool};
pub use query_builder::QueryBuilder;
pub use traits::{Crud, Model};
#[cfg(feature = "mysql")]
pub use transaction::with_mysql_nested_transaction;
#[cfg(feature = "postgres")]
pub use transaction::with_postgres_nested_transaction;
pub use transaction::{with_transaction, Transaction};

// 重新导出 derive 的所有公共 API（宏）
pub use error::{Result, SqlxPlusError};
pub use sqlxplus_derive::*;
