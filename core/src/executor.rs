use crate::db_pool::DbDriver;

/// 数据库执行器 trait，统一 DbPool 和 Transaction 的接口
/// 类似于 sqlx 的 Executor trait，允许 CRUD 方法同时支持连接池和事务
///
/// 这个 trait 提供了与 DbPool 相同的方法签名，让 CRUD 方法可以统一使用
///
/// 注意：此 trait 要求 `Send`，因为异步方法需要在不同线程之间传递 Future
pub trait DbExecutor: Send {
    /// 获取驱动类型
    fn driver(&self) -> DbDriver;

    /// 转换 SQL 占位符
    fn convert_sql(&self, sql: &str) -> String {
        self.driver().convert_placeholders(sql)
    }

    /// 获取 MySQL 连接池引用（用于 CRUD 方法）
    /// 对于 DbPool，返回实际的 Pool
    /// 对于 Transaction，返回 None（需要使用 mysql_transaction_ref）
    #[cfg(feature = "mysql")]
    fn mysql_pool(&self) -> Option<&sqlx::Pool<sqlx::MySql>>;

    /// 获取 MySQL 事务引用（用于在事务中执行 CRUD 方法）
    /// 对于 DbPool，返回 None
    /// 对于 Transaction，返回事务内部的连接
    #[cfg(feature = "mysql")]
    fn mysql_transaction_ref(&mut self) -> Option<&mut sqlx::Transaction<'static, sqlx::MySql>>;

    /// 获取 PostgreSQL 连接池引用
    #[cfg(feature = "postgres")]
    fn pg_pool(&self) -> Option<&sqlx::Pool<sqlx::Postgres>>;

    /// 获取 PostgreSQL 事务引用
    #[cfg(feature = "postgres")]
    fn postgres_transaction_ref(
        &mut self,
    ) -> Option<&mut sqlx::Transaction<'static, sqlx::Postgres>>;

    /// 获取 SQLite 连接池引用
    #[cfg(feature = "sqlite")]
    fn sqlite_pool(&self) -> Option<&sqlx::Pool<sqlx::Sqlite>>;

    /// 获取 SQLite 事务引用
    #[cfg(feature = "sqlite")]
    fn sqlite_transaction_ref(&mut self) -> Option<&mut sqlx::Transaction<'static, sqlx::Sqlite>>;
}
