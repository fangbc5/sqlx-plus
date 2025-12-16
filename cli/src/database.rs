use anyhow::{Context, Result};
use sqlx::{MySql, Pool, Postgres, Row, Sqlite};

/// 数据库驱动类型
#[derive(Debug, Clone, Copy)]
pub enum DbDriver {
    MySql,
    Postgres,
    Sqlite,
}

impl DbDriver {
    /// 从数据库 URL 推断驱动类型
    pub fn from_url(url: &str) -> Result<Self> {
        if url.starts_with("mysql://") || url.starts_with("mariadb://") {
            Ok(Self::MySql)
        } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            Ok(Self::Postgres)
        } else if url.starts_with("sqlite://") || url.starts_with("sqlite:") {
            Ok(Self::Sqlite)
        } else {
            anyhow::bail!("Unsupported database URL. Supported: mysql://, postgres://, sqlite://")
        }
    }
}

/// 数据库连接池（枚举类型以支持不同数据库）
pub enum DbPool {
    MySql(Pool<MySql>),
    Postgres(Pool<Postgres>),
    Sqlite(Pool<Sqlite>),
}

impl DbPool {
    /// 连接到数据库
    pub async fn connect(url: &str) -> Result<Self> {
        let driver = DbDriver::from_url(url)?;

        match driver {
            DbDriver::MySql => {
                let pool = sqlx::MySqlPool::connect(url)
                    .await
                    .context("Failed to connect to MySQL database")?;
                Ok(Self::MySql(pool))
            }
            DbDriver::Postgres => {
                let pool = sqlx::PgPool::connect(url)
                    .await
                    .context("Failed to connect to PostgreSQL database")?;
                Ok(Self::Postgres(pool))
            }
            DbDriver::Sqlite => {
                let pool = sqlx::SqlitePool::connect(url)
                    .await
                    .context("Failed to connect to SQLite database")?;
                Ok(Self::Sqlite(pool))
            }
        }
    }

    /// 获取驱动类型
    pub fn driver(&self) -> DbDriver {
        match self {
            Self::MySql(_) => DbDriver::MySql,
            Self::Postgres(_) => DbDriver::Postgres,
            Self::Sqlite(_) => DbDriver::Sqlite,
        }
    }

    /// 获取所有表名
    pub async fn get_tables(&self) -> Result<Vec<String>> {
        match self {
            Self::MySql(pool) => {
                let tables = sqlx::query_scalar::<_, String>(
                    "SELECT TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = DATABASE() AND TABLE_TYPE = 'BASE TABLE'"
                )
                .fetch_all(pool)
                .await
                .context("Failed to query MySQL tables")?;
                Ok(tables)
            }
            Self::Postgres(pool) => {
                let tables = sqlx::query_scalar::<_, String>(
                    "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename"
                )
                .fetch_all(pool)
                .await
                .context("Failed to query PostgreSQL tables")?;
                Ok(tables)
            }
            Self::Sqlite(pool) => {
                let tables = sqlx::query_scalar::<_, String>(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
                )
                .fetch_all(pool)
                .await
                .context("Failed to query SQLite tables")?;
                Ok(tables)
            }
        }
    }

    /// 获取表结构信息
    pub async fn get_table_info(&self, table_name: &str) -> Result<super::generator::TableInfo> {
        match self {
            Self::MySql(pool) => {
                let columns = sqlx::query_as::<_, ColumnRow>(
                    r#"
                    SELECT 
                        COLUMN_NAME as name,
                        CAST(COLUMN_TYPE AS CHAR) as sql_type,
                        IS_NULLABLE = 'YES' as nullable,
                        COLUMN_KEY = 'PRI' as is_pk,
                        CAST(COLUMN_DEFAULT AS CHAR) as default_value
                    FROM INFORMATION_SCHEMA.COLUMNS
                    WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?
                    ORDER BY ORDINAL_POSITION
                    "#,
                )
                .bind(table_name)
                .fetch_all(pool)
                .await
                .context("Failed to query MySQL table columns")?;

                Ok(super::generator::TableInfo {
                    name: table_name.to_string(),
                    columns: columns.into_iter().map(|c| c.into()).collect(),
                })
            }
            Self::Postgres(pool) => {
                let columns = sqlx::query_as::<_, ColumnRow>(
                    r#"
                    SELECT 
                        column_name as name,
                        data_type as sql_type,
                        is_nullable = 'YES' as nullable,
                        CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_pk,
                        column_default as default_value
                    FROM information_schema.columns c
                    LEFT JOIN (
                        SELECT ku.column_name
                        FROM information_schema.table_constraints tc
                        JOIN information_schema.key_column_usage ku
                            ON tc.constraint_name = ku.constraint_name
                            AND tc.table_schema = ku.table_schema
                        WHERE tc.constraint_type = 'PRIMARY KEY'
                            AND tc.table_name = $1
                    ) pk ON c.column_name = pk.column_name
                    WHERE c.table_schema = 'public' AND c.table_name = $1
                    ORDER BY c.ordinal_position
                    "#,
                )
                .bind(table_name)
                .fetch_all(pool)
                .await
                .context("Failed to query PostgreSQL table columns")?;

                Ok(super::generator::TableInfo {
                    name: table_name.to_string(),
                    columns: columns.into_iter().map(|c| c.into()).collect(),
                })
            }
            Self::Sqlite(pool) => {
                // SQLite 使用 PRAGMA table_info，需要手动解析结果
                let pragma_query = format!("PRAGMA table_info(\"{}\")", table_name);
                let rows = sqlx::query(&pragma_query)
                    .fetch_all(pool)
                    .await
                    .context("Failed to query SQLite table columns")?;

                let mut columns = Vec::new();
                for row in rows {
                    let _cid: i32 = row.get(0);
                    let name: String = row.get(1);
                    let sql_type: String = row.get(2);
                    let notnull: i32 = row.get(3);
                    let dflt_value: Option<String> = row.get(4);
                    let pk: i32 = row.get(5);

                    columns.push(super::generator::ColumnInfo {
                        name,
                        sql_type,
                        nullable: notnull == 0,
                        is_pk: pk > 0,
                        default: dflt_value,
                    });
                }

                Ok(super::generator::TableInfo {
                    name: table_name.to_string(),
                    columns,
                })
            }
        }
    }
}

/// MySQL/PostgreSQL 列信息行
#[derive(sqlx::FromRow)]
struct ColumnRow {
    name: String,
    sql_type: String,
    nullable: bool,
    is_pk: bool,
    #[sqlx(default)]
    default_value: Option<String>,
}

impl From<ColumnRow> for super::generator::ColumnInfo {
    fn from(row: ColumnRow) -> Self {
        Self {
            name: row.name,
            sql_type: row.sql_type,
            nullable: row.nullable,
            is_pk: row.is_pk,
            default: row.default_value,
        }
    }
}
