//! 数据库信息抽象层
//!
//! 提供统一的接口来访问不同数据库的特性，如占位符、标识符转义等。

use crate::db_pool::DbDriver;
use sqlx::Database;

/// 数据库信息 trait
///
/// 为不同的数据库类型提供统一的接口，用于获取数据库特定的信息，
/// 如占位符格式、标识符转义方式等。
///
/// # 实现要求
///
/// 每个数据库类型（`sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
/// 都需要实现此 trait，以提供数据库特定的行为。
pub trait DatabaseInfo: Database {
    /// 获取占位符字符串
    ///
    /// # 参数
    ///
    /// * `index` - 占位符的索引（从 0 开始）
    ///
    /// # 返回值
    ///
    /// * MySQL/SQLite: `"?"`
    /// * PostgreSQL: `"$1"`, `"$2"`, ... (index + 1)
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// assert_eq!(<sqlx::MySql as DatabaseInfo>::placeholder(0), "?");
    /// assert_eq!(<sqlx::Postgres as DatabaseInfo>::placeholder(0), "$1");
    /// assert_eq!(<sqlx::Postgres as DatabaseInfo>::placeholder(1), "$2");
    /// ```
    fn placeholder(index: usize) -> String;

    /// 转义 SQL 标识符（表名、列名等）
    ///
    /// # 参数
    ///
    /// * `name` - 需要转义的标识符名称
    ///
    /// # 返回值
    ///
    /// * MySQL: `` `name` ``
    /// * PostgreSQL/SQLite: `"name"`
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// assert_eq!(<sqlx::MySql as DatabaseInfo>::escape_identifier("user"), "`user`");
    /// assert_eq!(<sqlx::Postgres as DatabaseInfo>::escape_identifier("user"), "\"user\"");
    /// ```
    fn escape_identifier(name: &str) -> String;

    /// 获取数据库驱动类型
    ///
    /// # 返回值
    ///
    /// 对应的 `DbDriver` 枚举值
    fn get_driver() -> DbDriver;
}

// ========== MySQL 实现 ==========

#[cfg(feature = "mysql")]
impl DatabaseInfo for sqlx::MySql {
    fn placeholder(_index: usize) -> String {
        // MySQL 使用 ? 作为占位符，不依赖索引
        "?".to_string()
    }

    fn escape_identifier(name: &str) -> String {
        // MySQL 使用反引号转义标识符
        format!("`{}`", name)
    }

    fn get_driver() -> DbDriver {
        DbDriver::MySql
    }
}

// ========== PostgreSQL 实现 ==========

#[cfg(feature = "postgres")]
impl DatabaseInfo for sqlx::Postgres {
    fn placeholder(index: usize) -> String {
        // PostgreSQL 使用 $1, $2, ... 作为占位符
        format!("${}", index + 1)
    }

    fn escape_identifier(name: &str) -> String {
        // PostgreSQL 使用双引号转义标识符
        format!("\"{}\"", name)
    }

    fn get_driver() -> DbDriver {
        DbDriver::Postgres
    }
}

// ========== SQLite 实现 ==========

#[cfg(feature = "sqlite")]
impl DatabaseInfo for sqlx::Sqlite {
    fn placeholder(_index: usize) -> String {
        // SQLite 使用 ? 作为占位符，不依赖索引
        "?".to_string()
    }

    fn escape_identifier(name: &str) -> String {
        // SQLite 使用双引号转义标识符
        format!("\"{}\"", name)
    }

    fn get_driver() -> DbDriver {
        DbDriver::Sqlite
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "mysql")]
    #[test]
    fn test_mysql_placeholder() {
        assert_eq!(<sqlx::MySql as DatabaseInfo>::placeholder(0), "?");
        assert_eq!(<sqlx::MySql as DatabaseInfo>::placeholder(1), "?");
        assert_eq!(<sqlx::MySql as DatabaseInfo>::placeholder(100), "?");
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn test_mysql_escape_identifier() {
        assert_eq!(
            <sqlx::MySql as DatabaseInfo>::escape_identifier("user"),
            "`user`"
        );
        assert_eq!(
            <sqlx::MySql as DatabaseInfo>::escape_identifier("user_name"),
            "`user_name`"
        );
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn test_mysql_get_driver() {
        assert_eq!(<sqlx::MySql as DatabaseInfo>::get_driver(), DbDriver::MySql);
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn test_postgres_placeholder() {
        assert_eq!(<sqlx::Postgres as DatabaseInfo>::placeholder(0), "$1");
        assert_eq!(<sqlx::Postgres as DatabaseInfo>::placeholder(1), "$2");
        assert_eq!(<sqlx::Postgres as DatabaseInfo>::placeholder(2), "$3");
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn test_postgres_escape_identifier() {
        assert_eq!(
            <sqlx::Postgres as DatabaseInfo>::escape_identifier("user"),
            "\"user\""
        );
        assert_eq!(
            <sqlx::Postgres as DatabaseInfo>::escape_identifier("user_name"),
            "\"user_name\""
        );
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn test_postgres_get_driver() {
        assert_eq!(
            <sqlx::Postgres as DatabaseInfo>::get_driver(),
            DbDriver::Postgres
        );
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_placeholder() {
        assert_eq!(<sqlx::Sqlite as DatabaseInfo>::placeholder(0), "?");
        assert_eq!(<sqlx::Sqlite as DatabaseInfo>::placeholder(1), "?");
        assert_eq!(<sqlx::Sqlite as DatabaseInfo>::placeholder(100), "?");
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_escape_identifier() {
        assert_eq!(
            <sqlx::Sqlite as DatabaseInfo>::escape_identifier("user"),
            "\"user\""
        );
        assert_eq!(
            <sqlx::Sqlite as DatabaseInfo>::escape_identifier("user_name"),
            "\"user_name\""
        );
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_get_driver() {
        assert_eq!(
            <sqlx::Sqlite as DatabaseInfo>::get_driver(),
            DbDriver::Sqlite
        );
    }
}
