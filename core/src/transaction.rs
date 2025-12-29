use std::future::Future;
use std::pin::Pin;

use sqlx::{MySqlConnection, PgConnection, SqliteConnection};

use crate::db_pool::{DbDriver, DbPool};
use crate::error::{SqlxPlusError, Result};


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
    pub async fn begin(pool: &DbPool) -> Result<Self> {
        match pool.driver() {
            DbDriver::MySql => {
                Ok(Transaction::MySql(pool.mysql_pool().ok_or(SqlxPlusError::NoPoolAvailable)?.begin().await?))
            }
            DbDriver::Postgres => {
                Ok(Transaction::Postgres(pool.pg_pool().ok_or(SqlxPlusError::NoPoolAvailable)?.begin().await?))
            }
            DbDriver::Sqlite => {
                Ok(Transaction::Sqlite(pool.sqlite_pool().ok_or(SqlxPlusError::NoPoolAvailable)?.begin().await?))
            }
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

    pub fn as_mysql_executor(&mut self) -> &mut MySqlConnection {
        match self {
            Transaction::MySql(tx) => tx,
            _ => panic!("Transaction is not a MySQL transaction"),
        }
    }

    pub fn as_postgres_executor(&mut self) -> &mut PgConnection {
        match self {
            Transaction::Postgres(tx) => tx,
            _ => panic!("Transaction is not a PostgreSQL transaction"),
        }
    }

    pub fn as_sqlite_executor(&mut self) -> &mut SqliteConnection {
        match self {
            Transaction::Sqlite(tx) => tx,
            _ => panic!("Transaction is not a SQLite transaction"),
        }
    }

    #[cfg(feature = "mysql")]
    pub fn as_mysql_transaction(&mut self) -> &mut sqlx::Transaction<'tx, sqlx::MySql> {
        match self {
            Transaction::MySql(tx) => tx,
            _ => panic!("Transaction is not a MySQL transaction"),
        }
    }

    #[cfg(feature = "postgres")]
    pub fn as_postgres_transaction(&mut self) -> &mut sqlx::Transaction<'tx, sqlx::Postgres> {
        match self {
            Transaction::Postgres(tx) => tx,
            _ => panic!("Transaction is not a PostgreSQL transaction"),
        }
    }

    #[cfg(feature = "sqlite")]
    pub fn as_sqlite_transaction(&mut self) -> &mut sqlx::Transaction<'tx, sqlx::Sqlite> {
        match self {
            Transaction::Sqlite(tx) => tx,
            _ => panic!("Transaction is not a SQLite transaction"),
        }
    }

}

pub async fn with_transaction<F, T>(pool: &DbPool, f: F) -> crate::Result<T>
where
    F: for<'a> FnOnce(
        &'a mut Transaction<'_>,
    ) -> Pin<Box<dyn Future<Output = crate::Result<T>> + Send + 'a>>,
    T: Send,
{
    let mut tx = Transaction::begin(pool).await?;

    match f(&mut tx).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            // Explicitly rollback on error
            // (Transaction would auto-rollback on drop anyway, but this makes it clearer)
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

#[cfg(feature = "mysql")]
pub async fn with_nested_transaction<F, T>(
    tx: &mut Transaction<'_>,
    f: F,
) -> crate::Result<T>
where
    F: for<'a> FnOnce(&'a mut Transaction<'_>) -> Pin<Box<dyn Future<Output = crate::Result<T>> + Send + 'a>>,
    T: Send,
{
    // 获取事务的执行器
    // let executor = match tx {
    //     Transaction::MySql(tx) => ,
    //     Transaction::Postgres(tx) => tx.as_postgres_executor(),
    //     Transaction::Sqlite(tx) => tx.as_sqlite_executor(),
    // };
    // Create a savepoint
    sqlx::query("SAVEPOINT nested_tx")
        .execute(tx.as_mysql_executor())
        .await?;

    match f(tx).await {
        Ok(result) => {
            // Release savepoint (equivalent to commit)
            sqlx::query("RELEASE SAVEPOINT nested_tx")
                .execute(tx.as_mysql_executor())
                .await?;
            Ok(result)
        }
        Err(e) => {
            // Rollback to savepoint
            let _ = sqlx::query("ROLLBACK TO SAVEPOINT nested_tx")
                .execute(tx.as_mysql_executor())
                .await;
            Err(e)
        }
    }
}
