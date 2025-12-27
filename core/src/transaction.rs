use crate::db_pool::{DbDriver, DbPool};
use crate::error::{SqlxPlusError, Result};
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
            Box::pin(async move $body)
        })
    };
    // 匹配值形式：pool（会自动借用）
    ($pool:expr, |$tx:ident| async move $body:block) => {
        $pool.transaction(|$tx| {
            Box::pin(async move $body)
        })
    };
}

/// 数据库事务包装器
/// 自动处理提交和回滚
#[derive(Debug)]
pub enum Transaction<'tx> {
    #[cfg(feature = "mysql")]
    MySql(sqlx::Transaction<'tx, sqlx::MySql>),
    #[cfg(feature = "postgres")]
    Postgres(sqlx::Transaction<'tx, sqlx::Postgres>),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::Transaction<'tx, sqlx::Sqlite>),
}

impl<'tx> Transaction<'tx> {
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
                tx.commit().await.map_err(SqlxPlusError::DatabaseError)?;
                Ok(())
            }
            #[cfg(feature = "postgres")]
            Transaction::Postgres(tx) => {
                tx.commit().await.map_err(SqlxPlusError::DatabaseError)?;
                Ok(())
            }
            #[cfg(feature = "sqlite")]
            Transaction::Sqlite(tx) => {
                tx.commit().await.map_err(SqlxPlusError::DatabaseError)?;
                Ok(())
            }
        }
    }

    /// 回滚事务
    pub async fn rollback(self) -> Result<()> {
        match self {
            #[cfg(feature = "mysql")]
            Transaction::MySql(tx) => {
                tx.rollback().await.map_err(SqlxPlusError::DatabaseError)?;
                Ok(())
            }
            #[cfg(feature = "postgres")]
            Transaction::Postgres(tx) => {
                tx.rollback().await.map_err(SqlxPlusError::DatabaseError)?;
                Ok(())
            }
            #[cfg(feature = "sqlite")]
            Transaction::Sqlite(tx) => {
                tx.rollback().await.map_err(SqlxPlusError::DatabaseError)?;
                Ok(())
            }
        }
    }

    /// 获取 MySQL 事务引用（如果适用）
    #[cfg(feature = "mysql")]
    pub fn mysql_transaction(&mut self) -> Option<&mut sqlx::Transaction<'tx, sqlx::MySql>> {
        match self {
            Transaction::MySql(tx) => Some(tx),
            _ => None,
        }
    }

    /// 获取 PostgreSQL 事务引用（如果适用）
    #[cfg(feature = "postgres")]
    pub fn postgres_transaction(
        &mut self,
    ) -> Option<&mut sqlx::Transaction<'tx, sqlx::Postgres>> {
        match self {
            Transaction::Postgres(tx) => Some(tx),
            _ => None,
        }
    }

    /// 获取 SQLite 事务引用（如果适用）
    #[cfg(feature = "sqlite")]
    pub fn sqlite_transaction(&mut self) -> Option<&mut sqlx::Transaction<'tx, sqlx::Sqlite>> {
        match self {
            Transaction::Sqlite(tx) => Some(tx),
            _ => None,
        }
    }
}

impl<'tx> DbExecutor for Transaction<'tx> {
    fn driver(&self) -> DbDriver {
        self.driver()
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
        // 注意：这里需要将 'tx 转换为 'static
        // 由于 Transaction 是从 Arc<Pool> 创建的，实际上是安全的
        match self {
            Transaction::MySql(tx) => {
                // 使用 unsafe 转换，因为从 Arc<Pool> 创建的 transaction 实际上是 'static
                unsafe { Some(std::mem::transmute(tx)) }
            }
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
            Transaction::Postgres(tx) => {
                unsafe { Some(std::mem::transmute(tx)) }
            }
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
            Transaction::Sqlite(tx) => {
                unsafe { Some(std::mem::transmute(tx)) }
            }
            _ => None,
        }
    }
}

impl DbPool {
    /// 开始一个事务
    /// 
    /// 注意：返回的 Transaction 的生命周期绑定到 DbPool，但由于 DbPool 内部使用 Arc，
    /// 实际上事务可以安全地转换为 'static 生命周期
    pub async fn begin(&self) -> Result<Transaction<'static>> {
        match self.driver() {
            #[cfg(feature = "mysql")]
            DbDriver::MySql => {
                let pool = self.mysql_pool().ok_or(SqlxPlusError::NoPoolAvailable)?;
                let tx = pool.begin().await.map_err(SqlxPlusError::DatabaseError)?;
                // 从 Arc<Pool> 创建的 transaction 实际上是 'static
                Ok(Transaction::MySql(unsafe { std::mem::transmute(tx) }))
            }
            #[cfg(feature = "postgres")]
            DbDriver::Postgres => {
                let pool = self.pg_pool().ok_or(SqlxPlusError::NoPoolAvailable)?;
                let tx = pool.begin().await.map_err(SqlxPlusError::DatabaseError)?;
                Ok(Transaction::Postgres(unsafe { std::mem::transmute(tx) }))
            }
            #[cfg(feature = "sqlite")]
            DbDriver::Sqlite => {
                let pool = self.sqlite_pool().ok_or(SqlxPlusError::NoPoolAvailable)?;
                let tx = pool.begin().await.map_err(SqlxPlusError::DatabaseError)?;
                Ok(Transaction::Sqlite(unsafe { std::mem::transmute(tx) }))
            }
            #[allow(unreachable_patterns)]
            _ => Err(SqlxPlusError::NoPoolAvailable),
        }
    }

    /// 内部实现：处理生命周期绑定的 BoxFuture
    async fn transaction_boxed<F, T, E>(&self, f: F) -> std::result::Result<T, E>
    where
        for<'a> F: FnOnce(
            &'a mut Transaction<'static>,
        )
            -> Pin<Box<dyn Future<Output = std::result::Result<T, E>> + Send + 'a>>,
        E: From<SqlxPlusError>,
    {
        let mut tx = self.begin().await.map_err(E::from)?;

        match f(&mut tx).await {
            Ok(result) => {
                tx.commit().await.map_err(E::from)?;
                Ok(result)
            }
            Err(e) => {
                // 尝试回滚，但忽略回滚错误（因为原始错误更重要）
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
        for<'a> F: FnOnce(
            &'a mut Transaction<'static>,
        )
            -> Pin<Box<dyn Future<Output = std::result::Result<T, E>> + Send + 'a>>,
        E: From<SqlxPlusError>,
    {
        self.transaction_boxed(f).await
    }
}
