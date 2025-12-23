#[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
use sqlx::Pool;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbDriver {
    MySql,
    Postgres,
    Sqlite,
}

impl DbDriver {
    pub fn from_url(url: &str) -> Result<Self> {
        if url.starts_with("mysql://") || url.starts_with("mariadb://") {
            Ok(DbDriver::MySql)
        } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            Ok(DbDriver::Postgres)
        } else if url.starts_with("sqlite://") || url.starts_with("sqlite:") {
            Ok(DbDriver::Sqlite)
        } else {
            Err(DbPoolError::UnsupportedDatabase(url.to_string()))
        }
    }

    pub fn placeholder(&self, index: usize) -> String {
        match self {
            DbDriver::MySql | DbDriver::Sqlite => "?".to_string(),
            DbDriver::Postgres => format!("${}", index + 1),
        }
    }

    pub fn convert_placeholders(&self, sql: &str) -> String {
        match self {
            DbDriver::MySql | DbDriver::Sqlite => sql.to_string(),
            DbDriver::Postgres => {
                let mut result = String::with_capacity(sql.len());
                let mut index = 0;
                for ch in sql.chars() {
                    if ch == '?' {
                        index += 1;
                        result.push_str(&format!("${}", index));
                    } else {
                        result.push(ch);
                    }
                }
                result
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbPool {
    driver: DbDriver,
    #[cfg(feature = "mysql")]
    mysql: Option<Arc<Pool<sqlx::MySql>>>,
    #[cfg(feature = "postgres")]
    pg: Option<Arc<Pool<sqlx::Postgres>>>,
    #[cfg(feature = "sqlite")]
    sqlite: Option<Arc<Pool<sqlx::Sqlite>>>,
}

#[derive(Debug, Error)]
pub enum DbPoolError {
    #[error("Unsupported database URL: {0}")]
    UnsupportedDatabase(String),
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] sqlx::Error),
    #[error("No connection pool available for driver")]
    NoPoolAvailable,
}

pub type Result<T> = std::result::Result<T, DbPoolError>;

impl DbPool {
    /// 从数据库 URL 连接并创建 DbPool
    pub async fn connect(url: &str) -> Result<Self> {
        let driver = DbDriver::from_url(url)?;

        match driver {
            #[cfg(feature = "mysql")]
            DbDriver::MySql => {
                let pool = Pool::<sqlx::MySql>::connect(url).await?;
                Self::from_mysql_pool(Arc::new(pool))
            }
            #[cfg(feature = "postgres")]
            DbDriver::Postgres => {
                let pool = Pool::<sqlx::Postgres>::connect(url).await?;
                Self::from_postgres_pool(Arc::new(pool))
            }
            #[cfg(feature = "sqlite")]
            DbDriver::Sqlite => {
                let pool = Pool::<sqlx::Sqlite>::connect(url).await?;
                Self::from_sqlite_pool(Arc::new(pool))
            }
            #[allow(unreachable_patterns)]
            _ => Err(DbPoolError::UnsupportedDatabase(format!(
                "Unsupported database driver, only mysql, postgres, sqlite is supported, got: {:?}",
                driver
            ))),
        }
    }

    /// 从 MySQL Pool 创建 DbPool
    #[cfg(feature = "mysql")]
    pub fn from_mysql_pool(pool: Arc<Pool<sqlx::MySql>>) -> Result<Self> {
        Ok(Self {
            driver: DbDriver::MySql,
            mysql: Some(pool),
            #[cfg(feature = "postgres")]
            pg: None,
            #[cfg(feature = "sqlite")]
            sqlite: None,
        })
    }

    /// 从 PostgreSQL Pool 创建 DbPool
    #[cfg(feature = "postgres")]
    pub fn from_postgres_pool(pool: Arc<Pool<sqlx::Postgres>>) -> Result<Self> {
        Ok(Self {
            driver: DbDriver::Postgres,
            #[cfg(feature = "mysql")]
            mysql: None,
            pg: Some(pool),
            #[cfg(feature = "sqlite")]
            sqlite: None,
        })
    }

    /// 从 SQLite Pool 创建 DbPool
    #[cfg(feature = "sqlite")]
    pub fn from_sqlite_pool(pool: Arc<Pool<sqlx::Sqlite>>) -> Result<Self> {
        Ok(Self {
            driver: DbDriver::Sqlite,
            #[cfg(feature = "mysql")]
            mysql: None,
            #[cfg(feature = "postgres")]
            pg: None,
            sqlite: Some(pool),
        })
    }

    pub fn driver(&self) -> DbDriver {
        self.driver
    }

    #[cfg(feature = "mysql")]
    pub fn mysql_pool(&self) -> Option<&Pool<sqlx::MySql>> {
        self.mysql.as_deref()
    }

    #[cfg(feature = "postgres")]
    pub fn pg_pool(&self) -> Option<&Pool<sqlx::Postgres>> {
        self.pg.as_deref()
    }

    #[cfg(feature = "sqlite")]
    pub fn sqlite_pool(&self) -> Option<&Pool<sqlx::Sqlite>> {
        self.sqlite.as_deref()
    }

    pub fn convert_sql(&self, sql: &str) -> String {
        self.driver.convert_placeholders(sql)
    }

    pub async fn execute(&self, sql: &str) -> Result<u64> {
        let sql = self.convert_sql(sql);
        match self.driver {
            #[cfg(feature = "mysql")]
            DbDriver::MySql => {
                let pool = self.mysql.as_deref().ok_or(DbPoolError::NoPoolAvailable)?;
                let result = sqlx::query(&sql).execute(pool).await?;
                Ok(result.rows_affected())
            }
            #[cfg(feature = "postgres")]
            DbDriver::Postgres => {
                let pool = self.pg.as_deref().ok_or(DbPoolError::NoPoolAvailable)?;
                let result = sqlx::query(&sql).execute(pool).await?;
                Ok(result.rows_affected())
            }
            #[cfg(feature = "sqlite")]
            DbDriver::Sqlite => {
                let pool = self.sqlite.as_deref().ok_or(DbPoolError::NoPoolAvailable)?;
                let result = sqlx::query(&sql).execute(pool).await?;
                Ok(result.rows_affected())
            }
            #[allow(unreachable_patterns)]
            _ => Err(DbPoolError::NoPoolAvailable),
        }
    }

    pub async fn query_as<T>(&self, sql: &str) -> Result<Vec<T>>
    where
        T: Send
            + Unpin
            + for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>
            + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
    {
        let sql = self.convert_sql(sql);
        match self.driver {
            #[cfg(feature = "mysql")]
            DbDriver::MySql => {
                let pool = self.mysql.as_deref().ok_or(DbPoolError::NoPoolAvailable)?;
                let rows: Vec<T> = sqlx::query_as(&sql).fetch_all(pool).await?;
                Ok(rows)
            }
            #[cfg(feature = "postgres")]
            DbDriver::Postgres => {
                let pool = self.pg.as_deref().ok_or(DbPoolError::NoPoolAvailable)?;
                let rows: Vec<T> = sqlx::query_as(&sql).fetch_all(pool).await?;
                Ok(rows)
            }
            #[cfg(feature = "sqlite")]
            DbDriver::Sqlite => {
                let pool = self.sqlite.as_deref().ok_or(DbPoolError::NoPoolAvailable)?;
                let rows: Vec<T> = sqlx::query_as(&sql).fetch_all(pool).await?;
                Ok(rows)
            }
            #[allow(unreachable_patterns)]
            _ => Err(DbPoolError::NoPoolAvailable),
        }
    }
}
