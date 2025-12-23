use crate::db_pool::{DbDriver, DbPool, DbPoolError, Result};

/// 数据库事务包装器
/// 自动处理提交和回滚
#[derive(Debug)]
pub enum Transaction {
    #[cfg(feature = "mysql")]
    MySql(sqlx::Transaction<'static, sqlx::MySql>),
    #[cfg(feature = "postgres")]
    Postgres(sqlx::Transaction<'static, sqlx::Postgres>),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::Transaction<'static, sqlx::Sqlite>),
}

impl Transaction {
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
    pub fn mysql_transaction(&mut self) -> Option<&mut sqlx::Transaction<'static, sqlx::MySql>> {
        match self {
            Transaction::MySql(tx) => Some(tx),
            _ => None,
        }
    }

    /// 获取 PostgreSQL 事务引用（如果适用）
    #[cfg(feature = "postgres")]
    pub fn postgres_transaction(
        &mut self,
    ) -> Option<&mut sqlx::Transaction<'static, sqlx::Postgres>> {
        match self {
            Transaction::Postgres(tx) => Some(tx),
            _ => None,
        }
    }

    /// 获取 SQLite 事务引用（如果适用）
    #[cfg(feature = "sqlite")]
    pub fn sqlite_transaction(&mut self) -> Option<&mut sqlx::Transaction<'static, sqlx::Sqlite>> {
        match self {
            Transaction::Sqlite(tx) => Some(tx),
            _ => None,
        }
    }
}

impl DbPool {
    /// 开始一个事务
    pub async fn begin(&self) -> Result<Transaction> {
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

    /// 在事务中执行闭包函数
    /// 如果闭包返回 Ok，则自动提交事务
    /// 如果闭包返回 Err，则自动回滚事务
    pub async fn transaction<F, Fut, T, E>(&self, f: F) -> std::result::Result<T, E>
    where
        F: FnOnce(&mut Transaction) -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, E>> + Send,
        E: From<DbPoolError>,
    {
        let mut tx = self.begin().await.map_err(|e| E::from(e))?;

        match f(&mut tx).await {
            Ok(result) => {
                tx.commit().await.map_err(|e| E::from(e))?;
                Ok(result)
            }
            Err(e) => {
                // 尝试回滚，但忽略回滚错误（因为原始错误更重要）
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }
}
