use crate::db_pool::{DbDriver, DbPool, DbPoolError, Result};
use crate::executor::DbExecutor;
use std::future::Future;
use std::pin::Pin;

/// 宏：简化事务闭包的写法，自动处理 `Box::pin`
///
/// 使用示例：
/// ```ignore
/// // 使用引用
/// sqlxplus::transaction!(&pool, |tx| async move {
///     // 事务代码
///     Ok(42)
/// }).await?;
///
/// // 或直接使用值（会自动借用）
/// sqlxplus::transaction!(pool, |tx| async move {
///     // 事务代码
///     Ok(42)
/// }).await?;
/// ```
#[macro_export]
macro_rules! transaction {
    // 匹配引用形式：&pool
    (&$pool:expr, |$tx:ident| async move $body:block) => {
        
        $pool.transaction(|$tx| {
            Box::pin(async move {
                let result = $body;
                // 宏内部不处理提交/回滚，由 transaction_boxed 自动处理
                result
            })
        })
    };
    // 匹配值形式：pool（会自动借用）
    ($pool:expr, |$tx:ident| async move $body:block) => {
        $pool.transaction(|$tx| {
            Box::pin(async move {
                let result = $body;
                // 宏内部不处理提交/回滚，由 transaction_boxed 自动处理
                result
            })
        })
    };
}

/// 数据库事务包装器
/// 自动处理提交和回滚
#[derive(Debug)]
pub enum Transaction<'c> {
    #[cfg(feature = "mysql")]
    MySql(sqlx::Transaction<'c, sqlx::MySql>),
    #[cfg(feature = "postgres")]
    Postgres(sqlx::Transaction<'c, sqlx::Postgres>),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::Transaction<'c, sqlx::Sqlite>),
}

impl<'c> Transaction<'c> {
    /// 获取事务的驱动类型
    pub fn driver(&self) -> DbDriver {
        match self {
            #[cfg(feature = "mysql")]
            Transaction::MySql(_) => DbDriver::MySql,
            #[cfg(feature = "postgres")]
            Transaction::Postgres(_) => DbDriver::Postgres,
            #[cfg(feature = "sqlite")]
            Transaction::Sqlite(_) => DbDriver::Sqlite,
        }
    }

    /// 提交事务
    pub async fn commit(self) -> Result<()> {
        match self {
            #[cfg(feature = "mysql")]
            Transaction::MySql(tx) => {
                tx.commit().await.map_err(DbPoolError::ConnectionError)?;
                Ok(())
            }
            #[cfg(feature = "postgres")]
            Transaction::Postgres(tx) => {
                tx.commit().await.map_err(DbPoolError::ConnectionError)?;
                Ok(())
            }
            #[cfg(feature = "sqlite")]
            Transaction::Sqlite(tx) => {
                tx.commit().await.map_err(DbPoolError::ConnectionError)?;
                Ok(())
            }
        }
    }

    /// 回滚事务
    pub async fn rollback(self) -> Result<()> {
        match self {
            #[cfg(feature = "mysql")]
            Transaction::MySql(tx) => {
                tx.rollback().await.map_err(DbPoolError::ConnectionError)?;
                Ok(())
            }
            #[cfg(feature = "postgres")]
            Transaction::Postgres(tx) => {
                tx.rollback().await.map_err(DbPoolError::ConnectionError)?;
                Ok(())
            }
            #[cfg(feature = "sqlite")]
            Transaction::Sqlite(tx) => {
                tx.rollback().await.map_err(DbPoolError::ConnectionError)?;
                Ok(())
            }
        }
    }

    /// 获取 MySQL 事务引用（如果适用）
    #[cfg(feature = "mysql")]
    pub fn mysql_transaction(&mut self) -> Option<&mut sqlx::Transaction<'c, sqlx::MySql>> {
        match self {
            Transaction::MySql(tx) => Some(tx),
            _ => None,
        }
    }

    /// 获取 PostgreSQL 事务引用（如果适用）
    #[cfg(feature = "postgres")]
    pub fn postgres_transaction(&mut self) -> Option<&mut sqlx::Transaction<'c, sqlx::Postgres>> {
        match self {
            Transaction::Postgres(tx) => Some(tx),
            _ => None,
        }
    }

    /// 获取 SQLite 事务引用（如果适用）
    #[cfg(feature = "sqlite")]
    pub fn sqlite_transaction(&mut self) -> Option<&mut sqlx::Transaction<'c, sqlx::Sqlite>> {
        match self {
            Transaction::Sqlite(tx) => Some(tx),
            _ => None,
        }
    }
}

// 为 &mut Transaction 实现 DbExecutor，支持传递可变引用
impl<'c> DbExecutor for &mut Transaction<'c> {
    fn driver(&self) -> DbDriver {
        match self {
            Transaction::MySql(_) => DbDriver::MySql,
            Transaction::Postgres(_) => DbDriver::Postgres,
            Transaction::Sqlite(_) => DbDriver::Sqlite,
        }
    }

    fn convert_sql(&self, sql: &str) -> String {
        self.driver().convert_placeholders(sql)
    }

    #[cfg(feature = "mysql")]
    fn mysql_pool(&self) -> Option<&sqlx::Pool<sqlx::MySql>> {
        None
    }

    #[cfg(feature = "mysql")]
    fn mysql_transaction_ref(&mut self) -> Option<&mut sqlx::Transaction<'static, sqlx::MySql>> {
        // 使用 unsafe 将生命周期从 'c 转换为 'static
        // 这是安全的，因为 Transaction 在 drop 时会正确处理资源
        match self {
            Transaction::MySql(tx) => unsafe { Some(std::mem::transmute(tx)) },
            _ => None,
        }
    }

    #[cfg(feature = "postgres")]
    fn pg_pool(&self) -> Option<&sqlx::Pool<sqlx::Postgres>> {
        None
    }

    #[cfg(feature = "postgres")]
    fn postgres_transaction_ref(
        &mut self,
    ) -> Option<&mut sqlx::Transaction<'static, sqlx::Postgres>> {
        match self {
            Transaction::Postgres(tx) => unsafe { Some(std::mem::transmute(tx)) },
            _ => None,
        }
    }

    #[cfg(feature = "sqlite")]
    fn sqlite_pool(&self) -> Option<&sqlx::Pool<sqlx::Sqlite>> {
        None
    }

    #[cfg(feature = "sqlite")]
    fn sqlite_transaction_ref(&mut self) -> Option<&mut sqlx::Transaction<'static, sqlx::Sqlite>> {
        match self {
            Transaction::Sqlite(tx) => unsafe { Some(std::mem::transmute(tx)) },
            _ => None,
        }
    }
}

impl DbPool {
    /// 开始一个事务
    pub async fn begin(&self) -> Result<Transaction<'_>> {
        match self.driver() {
            #[cfg(feature = "mysql")]
            DbDriver::MySql => {
                let pool = self.mysql_pool().ok_or(DbPoolError::NoPoolAvailable)?;
                let tx = pool.begin().await.map_err(DbPoolError::ConnectionError)?;
                Ok(Transaction::MySql(tx))
            }
            #[cfg(feature = "postgres")]
            DbDriver::Postgres => {
                let pool = self.pg_pool().ok_or(DbPoolError::NoPoolAvailable)?;
                let tx = pool.begin().await.map_err(DbPoolError::ConnectionError)?;
                Ok(Transaction::Postgres(tx))
            }
            #[cfg(feature = "sqlite")]
            DbDriver::Sqlite => {
                let pool = self.sqlite_pool().ok_or(DbPoolError::NoPoolAvailable)?;
                let tx = pool.begin().await.map_err(DbPoolError::ConnectionError)?;
                Ok(Transaction::Sqlite(tx))
            }
            #[allow(unreachable_patterns)]
            _ => Err(DbPoolError::NoPoolAvailable),
        }
    }

    /// 内部实现：处理生命周期绑定的 BoxFuture
    async fn transaction_boxed<F, T, E>(&self, f: F) -> std::result::Result<T, E>
    where
        F: for<'c> FnOnce(
            &'c mut Transaction<'c>,
        )
            -> Pin<Box<dyn Future<Output = std::result::Result<T, E>> + Send + 'c>>,
        E: From<DbPoolError>,
    {
        let mut tx = self.begin().await.map_err(E::from)?;

        // 使用 unsafe 将生命周期从 '_ 转换为 'c
        // 这是安全的，因为事务的生命周期实际上由连接池管理
        let tx_ref: &mut Transaction<'_> = &mut tx;
        let tx_ref_unsafe: &mut Transaction<'static> = unsafe { std::mem::transmute(tx_ref) };

        match f(tx_ref_unsafe).await {
            Ok(result) => {
                // 闭包返回 Ok，自动提交事务
                tx.commit().await.map_err(E::from)?;
                Ok(result)
            }
            Err(e) => {
                // 闭包返回 Err，自动回滚事务
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }

    /// 在事务中执行闭包函数
    /// 如果闭包返回 Ok，则自动提交事务
    /// 如果闭包返回 Err，则自动回滚事务
    ///
    /// # 使用方式
    ///
    /// 方式1：直接使用（需要 `Box::pin`）：
    /// ```ignore
    /// pool.transaction(|tx| {
    ///     Box::pin(async move {
    ///         // 事务代码
    ///         Ok(42)
    ///     })
    /// }).await?;
    /// ```
    ///
    /// 方式2：使用宏（推荐，更简洁）：
    /// ```ignore
    /// sqlxplus::transaction!(pool, |tx| async move {
    ///     // 事务代码
    ///     Ok(42)
    /// }).await?;
    /// ```
    pub async fn transaction<F, T, E>(&self, f: F) -> std::result::Result<T, E>
    where
        for<'c> F: FnOnce(
            &'c mut Transaction<'c>,
        )
            -> Pin<Box<dyn Future<Output = std::result::Result<T, E>> + Send + 'c>>,
        E: From<DbPoolError>,
    {
        self.transaction_boxed(f).await
    }
}
