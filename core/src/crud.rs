use crate::error::{SqlxPlusError, Result};
use crate::query_builder::{BindValue, QueryBuilder};
use crate::traits::Model;
use sqlx::Row;

/// 主键 ID 类型
pub type Id = i64;

/// 辅助函数：将单个绑定值应用到查询中
/// 这是一个通用的绑定逻辑，通过宏来应用到不同的查询类型
macro_rules! apply_bind_value {
    ($query:expr, $bind:expr) => {
        match $bind {
            BindValue::String(s) => {
                $query = $query.bind(s);
            }
            BindValue::Int64(i) => {
                $query = $query.bind(i);
            }
            BindValue::Int32(i) => {
                $query = $query.bind(*i);
            }
            BindValue::Int16(i) => {
                $query = $query.bind(*i);
            }
            BindValue::Float64(f) => {
                $query = $query.bind(f);
            }
            BindValue::Float32(f) => {
                $query = $query.bind(*f);
            }
            BindValue::Bool(b) => {
                $query = $query.bind(b);
            }
            BindValue::Null => {
                $query = $query.bind(Option::<String>::None);
            }
        }
    };
}

/// 辅助函数：将绑定值应用到查询中（用于 query_as）
#[cfg(feature = "mysql")]
fn apply_binds_to_query_as_mysql<'q, M>(
    mut query: sqlx::query::QueryAs<'q, sqlx::MySql, M, sqlx::mysql::MySqlArguments>,
    binds: &'q [BindValue],
) -> sqlx::query::QueryAs<'q, sqlx::MySql, M, sqlx::mysql::MySqlArguments>
where
    M: for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>,
{
    for bind in binds {
        apply_bind_value!(query, bind);
    }
    query
}

/// 辅助函数：将绑定值应用到查询中（用于 query_as）
#[cfg(feature = "postgres")]
fn apply_binds_to_query_as_postgres<'q, M>(
    mut query: sqlx::query::QueryAs<'q, sqlx::Postgres, M, sqlx::postgres::PgArguments>,
    binds: &'q [BindValue],
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, M, sqlx::postgres::PgArguments>
where
    M: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>,
{
    for bind in binds {
        apply_bind_value!(query, bind);
    }
    query
}

/// 辅助函数：将绑定值应用到查询中（用于 query_as）
#[cfg(feature = "sqlite")]
fn apply_binds_to_query_as_sqlite<'q, M>(
    mut query: sqlx::query::QueryAs<'q, sqlx::Sqlite, M, sqlx::sqlite::SqliteArguments<'q>>,
    binds: &'q [BindValue],
) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, M, sqlx::sqlite::SqliteArguments<'q>>
where
    M: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    for bind in binds {
        apply_bind_value!(query, bind);
    }
    query
}

/// 辅助函数：将绑定值应用到查询中（用于 query，不返回特定类型）
#[cfg(feature = "mysql")]
fn apply_binds_to_query_mysql<'q>(
    mut query: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    binds: &'q [BindValue],
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    for bind in binds {
        apply_bind_value!(query, bind);
    }
    query
}

/// 辅助函数：将绑定值应用到查询中（用于 query，不返回特定类型）
#[cfg(feature = "postgres")]
fn apply_binds_to_query_postgres<'q>(
    mut query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    binds: &'q [BindValue],
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    for bind in binds {
        apply_bind_value!(query, bind);
    }
    query
}

/// 辅助函数：将绑定值应用到查询中（用于 query，不返回特定类型）
#[cfg(feature = "sqlite")]
fn apply_binds_to_query_sqlite<'q>(
    mut query: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    binds: &'q [BindValue],
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    for bind in binds {
        apply_bind_value!(query, bind);
    }
    query
}

/// 分页结果
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub size: u64,
    pub pages: u64,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, size: u64) -> Self {
        let pages = if size > 0 {
            (total + size - 1) / size
        } else {
            0
        };
        Self {
            items,
            total,
            page,
            size,
            pages,
        }
    }
}

/// 根据 ID 查找记录
pub async fn find_by_id<M, E>(
    executor: &mut E,
    id: impl for<'q> sqlx::Encode<'q, sqlx::MySql>
        + for<'q> sqlx::Encode<'q, sqlx::Postgres>
        + for<'q> sqlx::Encode<'q, sqlx::Sqlite>
        + sqlx::Type<sqlx::MySql>
        + sqlx::Type<sqlx::Postgres>
        + sqlx::Type<sqlx::Sqlite>
        + Send
        + Sync,
) -> Result<Option<M>>
where
    M: Model
        + for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>
        + Send
        + Unpin,
    E: crate::executor::DbExecutor,
{
    // 构建 SQL，如果指定了逻辑删除字段，自动过滤已删除的记录
    use crate::utils::escape_identifier;
    let driver = executor.driver();
    let escaped_table = escape_identifier(driver, M::TABLE);
    let escaped_pk = escape_identifier(driver, M::PK);
    let mut sql_str = format!("SELECT * FROM {} WHERE {} = ?", escaped_table, escaped_pk);
    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        let escaped_field = escape_identifier(driver, soft_delete_field);
        sql_str.push_str(&format!(" AND {} = 0", escaped_field));
    }
    let sql = executor.convert_sql(&sql_str);

    match executor.driver() {
        #[cfg(feature = "mysql")]
        crate::db_pool::DbDriver::MySql => {
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.mysql_transaction_ref() {
                match sqlx::query(&sql)
                    .bind(id)
                    .fetch_optional(&mut **tx_ref)
                    .await?
                {
                    Some(row) => Ok(Some(sqlx::FromRow::from_row(&row).map_err(|e| {
                        SqlxPlusError::DatabaseError(sqlx::Error::Decode(Box::new(
                            e,
                        )))
                    })?)),
                    None => Ok(None),
                }
            } else if let Some(pool_ref) = executor.mysql_pool() {
                match sqlx::query(&sql).bind(id).fetch_optional(pool_ref).await? {
                    Some(row) => Ok(Some(sqlx::FromRow::from_row(&row).map_err(|e| {
                        SqlxPlusError::DatabaseError(sqlx::Error::Decode(Box::new(
                            e,
                        )))
                    })?)),
                    None => Ok(None),
                }
            } else {
                Err(SqlxPlusError::NoPoolAvailable)
            }
        }
        #[cfg(feature = "postgres")]
        crate::db_pool::DbDriver::Postgres => {
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.postgres_transaction_ref() {
                match sqlx::query(&sql)
                    .bind(id)
                    .fetch_optional(&mut **tx_ref)
                    .await?
                {
                    Some(row) => Ok(Some(sqlx::FromRow::from_row(&row).map_err(|e| {
                        SqlxPlusError::DatabaseError(sqlx::Error::Decode(Box::new(
                            e,
                        )))
                    })?)),
                    None => Ok(None),
                }
            } else if let Some(pool_ref) = executor.pg_pool() {
                match sqlx::query(&sql).bind(id).fetch_optional(pool_ref).await? {
                    Some(row) => Ok(Some(sqlx::FromRow::from_row(&row).map_err(|e| {
                        SqlxPlusError::DatabaseError(sqlx::Error::Decode(Box::new(
                            e,
                        )))
                    })?)),
                    None => Ok(None),
                }
            } else {
                Err(SqlxPlusError::NoPoolAvailable)
            }
        }
        #[cfg(feature = "sqlite")]
        crate::db_pool::DbDriver::Sqlite => {
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.sqlite_transaction_ref() {
                match sqlx::query(&sql)
                    .bind(id)
                    .fetch_optional(&mut **tx_ref)
                    .await?
                {
                    Some(row) => Ok(Some(sqlx::FromRow::from_row(&row).map_err(|e| {
                        SqlxPlusError::DatabaseError(sqlx::Error::Decode(Box::new(
                            e,
                        )))
                    })?)),
                    None => Ok(None),
                }
            } else if let Some(pool_ref) = executor.sqlite_pool() {
                match sqlx::query(&sql).bind(id).fetch_optional(pool_ref).await? {
                    Some(row) => Ok(Some(sqlx::FromRow::from_row(&row).map_err(|e| {
                        SqlxPlusError::DatabaseError(sqlx::Error::Decode(Box::new(
                            e,
                        )))
                    })?)),
                    None => Ok(None),
                }
            } else {
                Err(SqlxPlusError::NoPoolAvailable)
            }
        }
        #[allow(unreachable_patterns)]
        _ => Err(SqlxPlusError::NoPoolAvailable),
    }
}

/// 根据多个 ID 查找记录
pub async fn find_by_ids<M, I, E>(executor: &mut E, ids: I) -> Result<Vec<M>>
where
    M: Model
        + for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>
        + Send
        + Unpin,
    I: IntoIterator + Send,
    I::Item: for<'q> sqlx::Encode<'q, sqlx::MySql>
        + for<'q> sqlx::Encode<'q, sqlx::Postgres>
        + for<'q> sqlx::Encode<'q, sqlx::Sqlite>
        + sqlx::Type<sqlx::MySql>
        + sqlx::Type<sqlx::Postgres>
        + sqlx::Type<sqlx::Sqlite>
        + Send
        + Sync
        + Clone,
    E: crate::executor::DbExecutor,
{
    // 将 IDs 收集到 Vec 中
    let ids_vec: Vec<_> = ids.into_iter().collect();

    // 如果 IDs 为空，直接返回空向量
    if ids_vec.is_empty() {
        return Ok(Vec::new());
    }

    // 构建 SQL，如果指定了逻辑删除字段，自动过滤已删除的记录
    use crate::utils::escape_identifier;
    let driver = executor.driver();
    let escaped_table = escape_identifier(driver, M::TABLE);
    let escaped_pk = escape_identifier(driver, M::PK);

    // 构建 IN 子句的占位符（使用 ? 占位符，convert_sql 会自动转换为对应数据库格式）
    let placeholders_str = (0..ids_vec.len())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");

    let mut sql_str = format!(
        "SELECT * FROM {} WHERE {} IN ({})",
        escaped_table, escaped_pk, placeholders_str
    );

    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        let escaped_field = escape_identifier(driver, soft_delete_field);
        sql_str.push_str(&format!(" AND {} = 0", escaped_field));
    }

    let sql = executor.convert_sql(&sql_str);

    match executor.driver() {
        #[cfg(feature = "mysql")]
        crate::db_pool::DbDriver::MySql => {
            let mut query = sqlx::query_as::<sqlx::MySql, M>(&sql);
            for id in &ids_vec {
                query = query.bind(id.clone());
            }
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.mysql_transaction_ref() {
                query
                    .fetch_all(&mut **tx_ref)
                    .await
                    .map_err(|e| SqlxPlusError::DatabaseError(e))
            } else if let Some(pool_ref) = executor.mysql_pool() {
                query
                    .fetch_all(pool_ref)
                    .await
                    .map_err(|e| SqlxPlusError::DatabaseError(e))
            } else {
                Err(SqlxPlusError::NoPoolAvailable)
            }
        }
        #[cfg(feature = "postgres")]
        crate::db_pool::DbDriver::Postgres => {
            let mut query = sqlx::query_as::<sqlx::Postgres, M>(&sql);
            for id in &ids_vec {
                query = query.bind(id.clone());
            }
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.postgres_transaction_ref() {
                query
                    .fetch_all(&mut **tx_ref)
                    .await
                    .map_err(|e| SqlxPlusError::DatabaseError(e))
            } else if let Some(pool_ref) = executor.pg_pool() {
                query
                    .fetch_all(pool_ref)
                    .await
                    .map_err(|e| SqlxPlusError::DatabaseError(e))
            } else {
                Err(SqlxPlusError::NoPoolAvailable)
            }
        }
        #[cfg(feature = "sqlite")]
        crate::db_pool::DbDriver::Sqlite => {
            let mut query = sqlx::query_as::<sqlx::Sqlite, M>(&sql);
            for id in &ids_vec {
                query = query.bind(id.clone());
            }
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.sqlite_transaction_ref() {
                query
                    .fetch_all(&mut **tx_ref)
                    .await
                    .map_err(|e| SqlxPlusError::DatabaseError(e))
            } else if let Some(pool_ref) = executor.sqlite_pool() {
                query
                    .fetch_all(pool_ref)
                    .await
                    .map_err(|e| SqlxPlusError::DatabaseError(e))
            } else {
                Err(SqlxPlusError::NoPoolAvailable)
            }
        }
        #[allow(unreachable_patterns)]
        _ => Err(SqlxPlusError::NoPoolAvailable),
    }
}

/// 插入记录（需要由 derive 宏生成具体的 SQL）
pub async fn insert<M, E>(_model: &M, _executor: &mut E) -> Result<Id>
where
    M: Model,
    E: crate::executor::DbExecutor,
{
    // 这个函数应该由 derive(CRUD) 宏生成具体实现
    // 这里提供一个占位实现
    Err(SqlxPlusError::DatabaseError(
        sqlx::Error::Configuration("insert() must be implemented by derive(CRUD) macro".into()),
    ))
}

/// 更新记录（需要由 derive 宏生成具体的 SQL）
pub async fn update<M, E>(_model: &M, _executor: &mut E) -> Result<()>
where
    M: Model,
    E: crate::executor::DbExecutor,
{
    // 这个函数应该由 derive(CRUD) 宏生成具体实现
    // 这里提供一个占位实现
    Err(SqlxPlusError::DatabaseError(
        sqlx::Error::Configuration("update() must be implemented by derive(CRUD) macro".into()),
    ))
}

/// 更新记录（Reset 语义：Option 字段为 None 时重置为数据库默认值）
///
/// 实际 SQL 逻辑由 `derive(CRUD)` 宏生成。此处仅作为占位实现，
/// 如果用户手动实现 `Crud` 而未提供对应实现，将在运行时报错提示。
pub async fn update_with_none<M, E>(_model: &M, _executor: &mut E) -> Result<()>
where
    M: Model,
    E: crate::executor::DbExecutor,
{
    Err(SqlxPlusError::DatabaseError(
        sqlx::Error::Configuration(
            "update_with_none() must be implemented by derive(CRUD) macro".into(),
        ),
    ))
}

/// 根据 ID 物理删除记录
pub async fn hard_delete_by_id<M, E>(
    executor: &mut E,
    id: impl for<'q> sqlx::Encode<'q, sqlx::MySql>
        + for<'q> sqlx::Encode<'q, sqlx::Postgres>
        + for<'q> sqlx::Encode<'q, sqlx::Sqlite>
        + sqlx::Type<sqlx::MySql>
        + sqlx::Type<sqlx::Postgres>
        + sqlx::Type<sqlx::Sqlite>
        + Send
        + Sync,
) -> Result<()>
where
    M: Model,
    E: crate::executor::DbExecutor,
{
    use crate::utils::escape_identifier;
    let driver = executor.driver();
    let escaped_table = escape_identifier(driver, M::TABLE);
    let escaped_pk = escape_identifier(driver, M::PK);
    let sql_str = format!("DELETE FROM {} WHERE {} = ?", escaped_table, escaped_pk);
    let sql = executor.convert_sql(&sql_str);

    match executor.driver() {
        #[cfg(feature = "mysql")]
        crate::db_pool::DbDriver::MySql => {
            if let Some(tx_ref) = executor.mysql_transaction_ref() {
                sqlx::query(&sql).bind(id).execute(&mut **tx_ref).await?;
            } else if let Some(pool_ref) = executor.mysql_pool() {
                sqlx::query(&sql).bind(id).execute(pool_ref).await?;
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[cfg(feature = "postgres")]
        crate::db_pool::DbDriver::Postgres => {
            if let Some(tx_ref) = executor.postgres_transaction_ref() {
                sqlx::query(&sql).bind(id).execute(&mut **tx_ref).await?;
            } else if let Some(pool_ref) = executor.pg_pool() {
                sqlx::query(&sql).bind(id).execute(pool_ref).await?;
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[cfg(feature = "sqlite")]
        crate::db_pool::DbDriver::Sqlite => {
            if let Some(tx_ref) = executor.sqlite_transaction_ref() {
                sqlx::query(&sql).bind(id).execute(&mut **tx_ref).await?;
            } else if let Some(pool_ref) = executor.sqlite_pool() {
                sqlx::query(&sql).bind(id).execute(pool_ref).await?;
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[allow(unreachable_patterns)]
        _ => return Err(SqlxPlusError::NoPoolAvailable),
    }

    Ok(())
}

/// 根据 ID 逻辑删除记录（将逻辑删除字段设置为 1）
pub async fn soft_delete_by_id<M, E>(
    executor: &mut E,
    id: impl for<'q> sqlx::Encode<'q, sqlx::MySql>
        + for<'q> sqlx::Encode<'q, sqlx::Postgres>
        + for<'q> sqlx::Encode<'q, sqlx::Sqlite>
        + sqlx::Type<sqlx::MySql>
        + sqlx::Type<sqlx::Postgres>
        + sqlx::Type<sqlx::Sqlite>
        + Send
        + Sync,
) -> Result<()>
where
    M: Model,
    E: crate::executor::DbExecutor,
{
    let soft_delete_field = M::SOFT_DELETE_FIELD.ok_or_else(|| {
        SqlxPlusError::DatabaseError(sqlx::Error::Configuration(
            format!(
                "Model {} does not have SOFT_DELETE_FIELD defined",
                std::any::type_name::<M>()
            )
            .into(),
        ))
    })?;

    use crate::utils::escape_identifier;
    let driver = executor.driver();
    let escaped_table = escape_identifier(driver, M::TABLE);
    let escaped_pk = escape_identifier(driver, M::PK);
    let escaped_field = escape_identifier(driver, soft_delete_field);
    let sql_str = format!(
        "UPDATE {} SET {} = 1 WHERE {} = ?",
        escaped_table, escaped_field, escaped_pk
    );
    let sql = executor.convert_sql(&sql_str);

    match executor.driver() {
        #[cfg(feature = "mysql")]
        crate::db_pool::DbDriver::MySql => {
            if let Some(tx_ref) = executor.mysql_transaction_ref() {
                sqlx::query(&sql).bind(id).execute(&mut **tx_ref).await?;
            } else if let Some(pool_ref) = executor.mysql_pool() {
                sqlx::query(&sql).bind(id).execute(pool_ref).await?;
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[cfg(feature = "postgres")]
        crate::db_pool::DbDriver::Postgres => {
            if let Some(tx_ref) = executor.postgres_transaction_ref() {
                sqlx::query(&sql).bind(id).execute(&mut **tx_ref).await?;
            } else if let Some(pool_ref) = executor.pg_pool() {
                sqlx::query(&sql).bind(id).execute(pool_ref).await?;
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[cfg(feature = "sqlite")]
        crate::db_pool::DbDriver::Sqlite => {
            if let Some(tx_ref) = executor.sqlite_transaction_ref() {
                sqlx::query(&sql).bind(id).execute(&mut **tx_ref).await?;
            } else if let Some(pool_ref) = executor.sqlite_pool() {
                sqlx::query(&sql).bind(id).execute(pool_ref).await?;
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[allow(unreachable_patterns)]
        _ => return Err(SqlxPlusError::NoPoolAvailable),
    }

    Ok(())
}

/// 安全查询所有记录（限制最多 1000 条）
/// 如果指定了 SOFT_DELETE_FIELD，自动过滤已删除的记录
pub async fn find_all<M, E>(executor: &mut E, builder: Option<QueryBuilder>) -> Result<Vec<M>>
where
    M: Model
        + for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>
        + Send
        + Unpin,
    E: crate::executor::DbExecutor,
{
    let driver = executor.driver();

    // 构建查询构建器
    use crate::utils::escape_identifier;
    let escaped_table = escape_identifier(driver, M::TABLE);
    let mut query_builder =
        builder.unwrap_or_else(|| QueryBuilder::new(format!("SELECT * FROM {}", escaped_table)));
    // 无论外部传入的 builder 使用了什么 base_sql，这里统一成基于模型表名的 SQL，保证风格与 find_by_id 一致
    query_builder = query_builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

    // 如果指定了逻辑删除字段，自动添加过滤条件（只查询未删除的记录）
    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        query_builder = query_builder.and_eq(soft_delete_field, 0);
    }

    // 限制最多 1000 条
    let mut sql = query_builder.into_sql(driver);
    sql.push_str(" LIMIT 1000");

    let binds = query_builder.binds().to_vec();

    let items: Vec<M> = match driver {
        #[cfg(feature = "mysql")]
        crate::db_pool::DbDriver::MySql => {
            let query = sqlx::query_as::<sqlx::MySql, M>(&sql);
            let query = apply_binds_to_query_as_mysql(query, &binds);
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.mysql_transaction_ref() {
                query.fetch_all(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.mysql_pool() {
                query.fetch_all(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[cfg(feature = "postgres")]
        crate::db_pool::DbDriver::Postgres => {
            let query = sqlx::query_as::<sqlx::Postgres, M>(&sql);
            let query = apply_binds_to_query_as_postgres(query, &binds);
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.postgres_transaction_ref() {
                query.fetch_all(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.pg_pool() {
                query.fetch_all(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[cfg(feature = "sqlite")]
        crate::db_pool::DbDriver::Sqlite => {
            let query = sqlx::query_as::<sqlx::Sqlite, M>(&sql);
            let query = apply_binds_to_query_as_sqlite(query, &binds);
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.sqlite_transaction_ref() {
                query.fetch_all(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.sqlite_pool() {
                query.fetch_all(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[allow(unreachable_patterns)]
        _ => {
            return Err(SqlxPlusError::UnsupportedDatabase(format!(
                "Unsupported database driver, only mysql, postgres, sqlite is supported, got: {:?}",
                driver
            )))
        }
    };

    Ok(items)
}

/// 查询单条记录（使用 QueryBuilder）
/// 如果指定了 SOFT_DELETE_FIELD，自动过滤已删除的记录
/// 自动添加 LIMIT 1 限制
pub async fn find_one<M, E>(executor: &mut E, builder: QueryBuilder) -> Result<Option<M>>
where
    M: Model
        + for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>
        + Send
        + Unpin,
    E: crate::executor::DbExecutor,
{
    let driver = executor.driver();

    // 构建查询构建器
    use crate::utils::escape_identifier;
    let escaped_table = escape_identifier(driver, M::TABLE);
    let mut query_builder = builder;
    // 无论外部传入的 builder 使用了什么 base_sql，这里统一成基于模型表名的 SQL
    query_builder = query_builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

    // 如果指定了逻辑删除字段，自动添加过滤条件（只查询未删除的记录）
    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        query_builder = query_builder.and_eq(soft_delete_field, 0);
    }

    // 自动添加 LIMIT 1
    let mut sql = query_builder.into_sql(driver);
    sql.push_str(" LIMIT 1");

    let binds = query_builder.binds().to_vec();

    let result: Option<M> = match driver {
        #[cfg(feature = "mysql")]
        crate::db_pool::DbDriver::MySql => {
            let query = sqlx::query_as::<sqlx::MySql, M>(&sql);
            let query = apply_binds_to_query_as_mysql(query, &binds);
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.mysql_transaction_ref() {
                query.fetch_optional(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.mysql_pool() {
                query.fetch_optional(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[cfg(feature = "postgres")]
        crate::db_pool::DbDriver::Postgres => {
            let query = sqlx::query_as::<sqlx::Postgres, M>(&sql);
            let query = apply_binds_to_query_as_postgres(query, &binds);
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.postgres_transaction_ref() {
                query.fetch_optional(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.pg_pool() {
                query.fetch_optional(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[cfg(feature = "sqlite")]
        crate::db_pool::DbDriver::Sqlite => {
            let query = sqlx::query_as::<sqlx::Sqlite, M>(&sql);
            let query = apply_binds_to_query_as_sqlite(query, &binds);
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.sqlite_transaction_ref() {
                query.fetch_optional(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.sqlite_pool() {
                query.fetch_optional(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[allow(unreachable_patterns)]
        _ => {
            return Err(SqlxPlusError::UnsupportedDatabase(format!(
                "Unsupported database driver, only mysql, postgres, sqlite is supported, got: {:?}",
                driver
            )))
        }
    };

    Ok(result)
}

/// 分页查询
pub async fn paginate<M, E>(
    executor: &mut E,
    mut builder: QueryBuilder,
    page: u64,
    size: u64,
) -> Result<Page<M>>
where
    M: Model
        + for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>
        + Send
        + Unpin,
    E: crate::executor::DbExecutor,
{
    let driver = executor.driver();
    let offset = (page - 1) * size;

    // 统一基础 SQL：始终从模型的表名出发，避免各处手写表名风格不一致
    use crate::utils::escape_identifier;
    let escaped_table = escape_identifier(driver, M::TABLE);
    builder = builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

    // 如果指定了逻辑删除字段，自动添加过滤条件（只查询未删除的记录）
    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        builder = builder.and_eq(soft_delete_field, 0);
    }

    let binds = builder.binds().to_vec();

    // 获取总数
    let count_sql = builder.clone().into_count_sql(driver);
    let total = match driver {
        #[cfg(feature = "mysql")]
        crate::db_pool::DbDriver::MySql => {
            let query = sqlx::query(&count_sql);
            let query = apply_binds_to_query_mysql(query, &binds);
            // 优先使用事务，获取不到再使用池
            let row = if let Some(tx_ref) = executor.mysql_transaction_ref() {
                query.fetch_one(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.mysql_pool() {
                query.fetch_one(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            };
            row.get::<i64, _>(0) as u64
        }
        #[cfg(feature = "postgres")]
        crate::db_pool::DbDriver::Postgres => {
            let query = sqlx::query(&count_sql);
            let query = apply_binds_to_query_postgres(query, &binds);
            // 优先使用事务，获取不到再使用池
            let row = if let Some(tx_ref) = executor.postgres_transaction_ref() {
                query.fetch_one(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.pg_pool() {
                query.fetch_one(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            };
            row.get::<i64, _>(0) as u64
        }
        #[cfg(feature = "sqlite")]
        crate::db_pool::DbDriver::Sqlite => {
            let query = sqlx::query(&count_sql);
            let query = apply_binds_to_query_sqlite(query, &binds);
            // 优先使用事务，获取不到再使用池
            let row = if let Some(tx_ref) = executor.sqlite_transaction_ref() {
                query.fetch_one(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.sqlite_pool() {
                query.fetch_one(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            };
            row.get::<i64, _>(0) as u64
        }
        #[allow(unreachable_patterns)]
        _ => return Err(SqlxPlusError::NoPoolAvailable),
    };

    // 获取分页数据
    let data_sql = builder.clone().into_paginated_sql(driver, size, offset);
    let items: Vec<M> = match driver {
        #[cfg(feature = "mysql")]
        crate::db_pool::DbDriver::MySql => {
            let query = sqlx::query_as::<sqlx::MySql, M>(&data_sql);
            let query = apply_binds_to_query_as_mysql(query, &binds);
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.mysql_transaction_ref() {
                query.fetch_all(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.mysql_pool() {
                query.fetch_all(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[cfg(feature = "postgres")]
        crate::db_pool::DbDriver::Postgres => {
            let query = sqlx::query_as::<sqlx::Postgres, M>(&data_sql);
            let query = apply_binds_to_query_as_postgres(query, &binds);
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.postgres_transaction_ref() {
                query.fetch_all(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.pg_pool() {
                query.fetch_all(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[cfg(feature = "sqlite")]
        crate::db_pool::DbDriver::Sqlite => {
            let query = sqlx::query_as::<sqlx::Sqlite, M>(&data_sql);
            let query = apply_binds_to_query_as_sqlite(query, &binds);
            // 优先使用事务，获取不到再使用池
            if let Some(tx_ref) = executor.sqlite_transaction_ref() {
                query.fetch_all(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.sqlite_pool() {
                query.fetch_all(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            }
        }
        #[allow(unreachable_patterns)]
        _ => {
            return Err(SqlxPlusError::UnsupportedDatabase(format!(
                "Unsupported database driver, only mysql, postgres, sqlite is supported, got: {:?}",
                driver
            )))
        }
    };

    Ok(Page::new(items, total, page, size))
}

/// 统计记录数量（使用 QueryBuilder）
/// 如果指定了 SOFT_DELETE_FIELD，自动过滤已删除的记录
pub async fn count<M, E>(executor: &mut E, builder: QueryBuilder) -> Result<u64>
where
    M: Model,
    E: crate::executor::DbExecutor,
{
    let driver = executor.driver();

    // 统一基础 SQL：始终从模型的表名出发
    use crate::utils::escape_identifier;
    let escaped_table = escape_identifier(driver, M::TABLE);
    let mut builder = builder;
    builder = builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

    // 如果指定了逻辑删除字段，自动添加过滤条件（只查询未删除的记录）
    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        builder = builder.and_eq(soft_delete_field, 0);
    }

    let binds = builder.binds().to_vec();

    // 获取总数
    let count_sql = builder.into_count_sql(driver);
    let total = match driver {
        #[cfg(feature = "mysql")]
        crate::db_pool::DbDriver::MySql => {
            let query = sqlx::query(&count_sql);
            let query = apply_binds_to_query_mysql(query, &binds);
            // 优先使用事务，获取不到再使用池
            let row = if let Some(tx_ref) = executor.mysql_transaction_ref() {
                query.fetch_one(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.mysql_pool() {
                query.fetch_one(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            };
            row.get::<i64, _>(0) as u64
        }
        #[cfg(feature = "postgres")]
        crate::db_pool::DbDriver::Postgres => {
            let query = sqlx::query(&count_sql);
            let query = apply_binds_to_query_postgres(query, &binds);
            // 优先使用事务，获取不到再使用池
            let row = if let Some(tx_ref) = executor.postgres_transaction_ref() {
                query.fetch_one(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.pg_pool() {
                query.fetch_one(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            };
            row.get::<i64, _>(0) as u64
        }
        #[cfg(feature = "sqlite")]
        crate::db_pool::DbDriver::Sqlite => {
            let query = sqlx::query(&count_sql);
            let query = apply_binds_to_query_sqlite(query, &binds);
            // 优先使用事务，获取不到再使用池
            let row = if let Some(tx_ref) = executor.sqlite_transaction_ref() {
                query.fetch_one(&mut **tx_ref).await?
            } else if let Some(pool_ref) = executor.sqlite_pool() {
                query.fetch_one(pool_ref).await?
            } else {
                return Err(SqlxPlusError::NoPoolAvailable);
            };
            row.get::<i64, _>(0) as u64
        }
        #[allow(unreachable_patterns)]
        _ => return Err(SqlxPlusError::NoPoolAvailable),
    };

    Ok(total)
}
