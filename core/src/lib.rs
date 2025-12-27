pub mod crud;
pub mod db_pool;
pub mod executor;
pub mod macros_api;
pub mod query_builder;
pub mod error;
pub mod traits;
pub mod transaction;
pub mod utils;

pub use crud::{Id, Page};
pub use db_pool::{DbDriver, DbPool};
pub use query_builder::QueryBuilder;
pub use traits::{Crud, Model};
pub use transaction::Transaction;

// 重新导出 derive 的所有公共 API（宏）
pub use sqlxplus_derive::*;
pub use error::{SqlxPlusError, Result};