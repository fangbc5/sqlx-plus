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
                // 从当前 search_path 中查询表
                // 使用 pg_catalog.pg_tables 和 current_schemas() 来获取当前 search_path 中的表
                let tables = sqlx::query_scalar::<_, String>(
                    r#"
                    SELECT tablename 
                    FROM pg_catalog.pg_tables 
                    WHERE schemaname = ANY(current_schemas(false))
                    ORDER BY tablename
                    "#
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
                // 先获取列信息
                let columns_query = sqlx::query_as::<_, ColumnRow>(
                    r#"
                    SELECT 
                        c.COLUMN_NAME as name,
                        CAST(c.COLUMN_TYPE AS CHAR) as sql_type,
                        c.IS_NULLABLE = 'YES' as nullable,
                        c.COLUMN_KEY = 'PRI' as is_pk,
                        CAST(c.COLUMN_DEFAULT AS CHAR) as default_value,
                        c.EXTRA LIKE '%auto_increment%' as auto_increment,
                        CAST(c.COLUMN_COMMENT AS CHAR) as comment
                    FROM INFORMATION_SCHEMA.COLUMNS c
                    WHERE c.TABLE_SCHEMA = DATABASE() AND c.TABLE_NAME = ?
                    ORDER BY c.ORDINAL_POSITION
                    "#,
                )
                .bind(table_name)
                .fetch_all(pool)
                .await
                .context("Failed to query MySQL table columns")?;

                // 获取索引信息
                let indexes_query = sqlx::query(
                    r#"
                    SELECT 
                        COLUMN_NAME,
                        CASE WHEN NON_UNIQUE = 0 THEN true ELSE false END as is_unique,
                        CASE WHEN NON_UNIQUE = 1 THEN true ELSE false END as has_index
                    FROM INFORMATION_SCHEMA.STATISTICS
                    WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?
                    AND INDEX_NAME != 'PRIMARY'
                    "#,
                )
                .bind(table_name)
                .fetch_all(pool)
                .await
                .context("Failed to query MySQL indexes")?;

                // 构建索引映射（一个字段可能有多个索引，优先 unique）
                let mut index_map: std::collections::HashMap<String, (bool, bool)> = std::collections::HashMap::new();
                for row in indexes_query {
                    let col_name: String = row.get(0);
                    let is_unique: bool = row.get(1);
                    let has_index: bool = row.get(2);
                    // 如果字段已经有 unique 索引，保持；否则更新
                    let entry = index_map.entry(col_name).or_insert((false, false));
                    if is_unique {
                        entry.0 = true;
                    }
                    if has_index {
                        entry.1 = true;
                    }
                }

                // 合并列信息和索引信息
                let mut columns = Vec::new();
                for mut col_row in columns_query {
                    if let Some((is_unique, has_index)) = index_map.get(&col_row.name) {
                        col_row.is_unique = *is_unique;
                        col_row.has_index = *has_index;
                    }
                    columns.push(col_row.into());
                }

                // 获取表注释（MySQL）
                let table_comment = sqlx::query_scalar::<_, Option<String>>(
                    r#"
                    SELECT CAST(TABLE_COMMENT AS CHAR)
                    FROM INFORMATION_SCHEMA.TABLES
                    WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?
                    "#,
                )
                .bind(table_name)
                .fetch_optional(pool)
                .await
                .context("Failed to query MySQL table comment")?
                .flatten();

                Ok(super::generator::TableInfo {
                    name: table_name.to_string(),
                    columns,
                    table_comment,
                })
            }
            Self::Postgres(pool) => {
                // 先获取列信息
                let columns_query = sqlx::query_as::<_, ColumnRow>(
                    r#"
                    SELECT 
                        c.column_name as name,
                        CASE 
                            WHEN c.character_maximum_length IS NOT NULL THEN 
                                c.data_type || '(' || c.character_maximum_length::text || ')'
                            ELSE c.data_type
                        END as sql_type,
                        c.is_nullable = 'YES' as nullable,
                        CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_pk,
                        c.column_default as default_value,
                        CASE WHEN c.column_default LIKE 'nextval%' THEN true ELSE false END as auto_increment,
                        COALESCE(pgd.description, NULL) as comment
                    FROM information_schema.columns c
                    LEFT JOIN (
                        SELECT ku.column_name
                        FROM information_schema.table_constraints tc
                        JOIN information_schema.key_column_usage ku
                            ON tc.constraint_name = ku.constraint_name
                            AND tc.table_schema = ku.table_schema
                        WHERE tc.constraint_type = 'PRIMARY KEY'
                            AND tc.table_name = $1
                            AND tc.table_schema = ANY(current_schemas(false))
                    ) pk ON c.column_name = pk.column_name
                    LEFT JOIN pg_catalog.pg_statio_all_tables st ON st.relname = c.table_name 
                        AND st.schemaname = ANY(current_schemas(false))
                    LEFT JOIN pg_catalog.pg_description pgd ON pgd.objoid = st.relid 
                        AND pgd.objsubid = c.ordinal_position
                    WHERE c.table_schema = ANY(current_schemas(false)) AND c.table_name = $1
                    ORDER BY c.ordinal_position
                    "#,
                )
                .bind(table_name)
                .fetch_all(pool)
                .await
                .context("Failed to query PostgreSQL table columns")?;

                // 获取索引信息
                let indexes_query = sqlx::query(
                    r#"
                    SELECT 
                        a.attname as column_name,
                        i.indisunique as is_unique,
                        CASE WHEN i.indisunique THEN false ELSE true END as has_index
                    FROM pg_index i
                    JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
                    JOIN pg_class t ON t.oid = i.indrelid
                    WHERE t.relname = $1 AND i.indisprimary = false
                    "#,
                )
                .bind(table_name)
                .fetch_all(pool)
                .await
                .context("Failed to query PostgreSQL indexes")?;

                // 构建索引映射（一个字段可能有多个索引，优先 unique）
                let mut index_map: std::collections::HashMap<String, (bool, bool)> = std::collections::HashMap::new();
                for row in indexes_query {
                    let col_name: String = row.get(0);
                    let is_unique: bool = row.get(1);
                    let has_index: bool = row.get(2);
                    // 如果字段已经有 unique 索引，保持；否则更新
                    let entry = index_map.entry(col_name).or_insert((false, false));
                    if is_unique {
                        entry.0 = true;
                    }
                    if has_index {
                        entry.1 = true;
                    }
                }

                // 合并列信息和索引信息
                let mut columns = Vec::new();
                for mut col_row in columns_query {
                    if let Some((is_unique, has_index)) = index_map.get(&col_row.name) {
                        col_row.is_unique = *is_unique;
                        col_row.has_index = *has_index;
                    }
                    // 处理注释，如果为空字符串则设为 None
                    if let Some(ref comment) = col_row.comment {
                        if comment.is_empty() {
                            col_row.comment = None;
                        }
                    }
                    columns.push(col_row.into());
                }

                // 获取表注释（PostgreSQL）
                // 先从 pg_tables 获取表所在的 schema，然后查询注释
                let table_comment = sqlx::query_scalar::<_, Option<String>>(
                    r#"
                    SELECT obj_description(c.oid, 'pg_class') as comment
                    FROM pg_class c
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relname = $1
                        AND n.nspname = (
                            SELECT schemaname 
                            FROM pg_tables 
                            WHERE tablename = $1 
                            LIMIT 1
                        )
                    LIMIT 1
                    "#,
                )
                .bind(table_name)
                .bind(table_name)
                .fetch_optional(pool)
                .await
                .context("Failed to query PostgreSQL table comment")?
                .flatten();

                Ok(super::generator::TableInfo {
                    name: table_name.to_string(),
                    columns,
                    table_comment,
                })
            }
            Self::Sqlite(pool) => {
                // SQLite 使用 PRAGMA table_info，需要手动解析结果
                let pragma_query = format!("PRAGMA table_info(\"{}\")", table_name);
                let rows = sqlx::query(&pragma_query)
                    .fetch_all(pool)
                    .await
                    .context("Failed to query SQLite table columns")?;

                // SQLite 索引信息需要通过 sqlite_master 查询
                // 注意：SQLite 的索引查询比较复杂，这里简化处理
                let indexes_query = sqlx::query(
                    r#"
                    SELECT 
                        sql
                    FROM sqlite_master
                    WHERE type = 'index' AND tbl_name = ? AND name NOT LIKE 'sqlite_%'
                    "#,
                )
                .bind(table_name)
                .fetch_all(pool)
                .await
                .context("Failed to query SQLite indexes")?;

                // 构建索引映射（简化处理，从 SQL 中解析）
                let mut index_map: std::collections::HashMap<String, (bool, bool)> = std::collections::HashMap::new();
                for row in indexes_query {
                    let sql: Option<String> = row.get(0);
                    if let Some(sql) = sql {
                        // 简单解析：如果包含 UNIQUE 则是唯一索引
                        let is_unique = sql.to_uppercase().contains("UNIQUE");
                        // 从 SQL 中提取列名（简化处理）
                        if let Some(start) = sql.find('(') {
                            if let Some(end) = sql.find(')') {
                                let cols_str = &sql[start + 1..end];
                                for col in cols_str.split(',') {
                                    let col_name = col.trim().trim_matches('"').trim_matches('\'').to_string();
                                    let entry = index_map.entry(col_name).or_insert((false, false));
                                    if is_unique {
                                        entry.0 = true;
                                    } else {
                                        entry.1 = true;
                                    }
                                }
                            }
                        }
                    }
                }

                let mut columns = Vec::new();
                for row in rows {
                    let _cid: i32 = row.get(0);
                    let name: String = row.get(1);
                    let sql_type: String = row.get(2);
                    let notnull: i32 = row.get(3);
                    let dflt_value: Option<String> = row.get(4);
                    let pk: i32 = row.get(5);

                    let (is_unique, has_index) = index_map.get(&name).copied().unwrap_or((false, false));
                    let length = extract_length_from_sql_type(&sql_type);

                    columns.push(super::generator::ColumnInfo {
                        name,
                        sql_type,
                        nullable: notnull == 0,
                        is_pk: pk > 0,
                        default: dflt_value,
                        auto_increment: false, // SQLite 不支持 AUTO_INCREMENT
                        is_unique,
                        has_index,
                        comment: None, // SQLite 不支持注释
                        length,
                    });
                }

                // SQLite 不支持表注释
                Ok(super::generator::TableInfo {
                    name: table_name.to_string(),
                    columns,
                    table_comment: None,
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
    #[sqlx(default)]
    auto_increment: bool,
    #[sqlx(default)]
    is_unique: bool,
    #[sqlx(default)]
    has_index: bool,
    #[sqlx(default)]
    comment: Option<String>,
}

impl From<ColumnRow> for super::generator::ColumnInfo {
    fn from(row: ColumnRow) -> Self {
        // 从 SQL 类型中提取长度（如 VARCHAR(255) -> 255）
        let length = extract_length_from_sql_type(&row.sql_type);
        
        Self {
            name: row.name,
            sql_type: row.sql_type,
            nullable: row.nullable,
            is_pk: row.is_pk,
            default: row.default_value,
            auto_increment: row.auto_increment,
            is_unique: row.is_unique,
            has_index: row.has_index,
            comment: row.comment,
            length,
        }
    }
}

/// 从 SQL 类型中提取长度（如 VARCHAR(255) -> 255）
fn extract_length_from_sql_type(sql_type: &str) -> Option<u32> {
    // 匹配类似 VARCHAR(255), CHAR(10) 等格式
    if let Some(start) = sql_type.find('(') {
        if let Some(end) = sql_type.find(')') {
            if let Ok(length) = sql_type[start + 1..end].trim().parse::<u32>() {
                return Some(length);
            }
        }
    }
    None
}
