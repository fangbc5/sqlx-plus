use crate::database_info::DatabaseInfo;
use crate::error::{Result, SqlxPlusError};
use crate::query_builder::{BindValue, QueryBuilder};
use crate::traits::Model;
use crate::utils::escape_identifier;
use sqlx::{Database, Row};

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

/// 根据 ID 查找单条记录（泛型版本）
///
/// 这是统一的泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
///
/// # 类型参数
///
/// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
/// * `M` - 模型类型，必须实现 `Model` trait 和对应数据库的 `FromRow`
/// * `E` - 执行器类型，可以是连接池或事务
///
/// # 参数
///
/// * `executor` - 数据库执行器（连接池或事务）
/// * `id` - 主键 ID 值
///
/// # 返回值
///
/// 如果找到记录，返回 `Ok(Some(M))`；如果未找到，返回 `Ok(None)`
///
/// # 示例
///
/// ```rust,ignore
/// use sqlxplus::{DatabaseInfo, crud};
///
/// // MySQL
/// let user = crud::find_by_id::<sqlx::MySql, User, _>(pool, 1).await?;
///
/// // PostgreSQL
/// let user = crud::find_by_id::<sqlx::Postgres, User, _>(pool, 1).await?;
///
/// // SQLite
/// let user = crud::find_by_id::<sqlx::Sqlite, User, _>(pool, 1).await?;
/// ```
pub async fn find_by_id<'e, 'c: 'e, DB, M, E>(
    executor: E,
    id: impl for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync,
) -> Result<Option<M>>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: Model + for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
    E: sqlx::Executor<'c, Database = DB> + Send,
{
    // 使用 DatabaseInfo trait 获取数据库特定信息
    let escaped_table = DB::escape_identifier(M::TABLE);
    let escaped_pk = DB::escape_identifier(M::PK);
    let placeholder = DB::placeholder(0);

    let sql_str = if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        let escaped_field = DB::escape_identifier(soft_delete_field);
        format!(
            "SELECT * FROM {} WHERE {} = {} AND {} = 0",
            escaped_table, escaped_pk, placeholder, escaped_field
        )
    } else {
        format!(
            "SELECT * FROM {} WHERE {} = {}",
            escaped_table, escaped_pk, placeholder
        )
    };

    // 执行查询 - sqlx::query 可以从 executor 推断数据库类型
    match sqlx::query(&sql_str)
        .bind(id)
        .fetch_optional(executor)
        .await?
    {
        Some(row) => Ok(Some(sqlx::FromRow::from_row(&row).map_err(|e| {
            SqlxPlusError::DatabaseError(sqlx::Error::Decode(
                Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>
            ))
        })?)),
        None => Ok(None),
    }
}

// 注意：find_by_id_mysql, find_by_id_postgres, find_by_id_sqlite 等兼容层函数已移除
// 现在直接使用泛型版本的 find_by_id<DB, M, E>
// trait 中的方法直接调用泛型版本，不再需要这些中间函数

/// 宏：生成数据库特定版本的 find_by_ids 函数
macro_rules! impl_find_by_ids_for_db {
    (
        $feature:literal,
        $db_type:ty,
        $row_type:ty,
        $driver:expr,
        $fn_name:ident,
        $placeholder_gen:expr
    ) => {
        #[cfg(feature = $feature)]
        pub async fn $fn_name<'e, 'c: 'e, M, I, E>(executor: E, ids: I) -> Result<Vec<M>>
        where
            M: Model + for<'r> sqlx::FromRow<'r, $row_type> + Send + Unpin,
            I: IntoIterator + Send,
            I::Item:
                for<'q> sqlx::Encode<'q, $db_type> + sqlx::Type<$db_type> + Send + Sync + Clone,
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            let ids_vec: Vec<_> = ids.into_iter().collect();
            if ids_vec.is_empty() {
                return Ok(Vec::new());
            }

            let escaped_table = escape_identifier($driver, M::TABLE);
            let escaped_pk = escape_identifier($driver, M::PK);
            let placeholders_str = ($placeholder_gen)(ids_vec.len());
            let mut sql_str = format!(
                "SELECT * FROM {} WHERE {} IN ({})",
                escaped_table, escaped_pk, placeholders_str
            );
            if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
                let escaped_field = escape_identifier($driver, soft_delete_field);
                sql_str.push_str(&format!(" AND {} = 0", escaped_field));
            }

            let mut query = sqlx::query_as::<$db_type, M>(&sql_str);
            for id in &ids_vec {
                query = query.bind(id.clone());
            }
            query
                .fetch_all(executor)
                .await
                .map_err(|e| SqlxPlusError::DatabaseError(e))
        }
    };
}

// 使用宏生成不同数据库版本的 find_by_ids 函数
impl_find_by_ids_for_db!(
    "mysql",
    sqlx::MySql,
    sqlx::mysql::MySqlRow,
    crate::db_pool::DbDriver::MySql,
    find_by_ids_mysql,
    |len| (0..len).map(|_| "?").collect::<Vec<_>>().join(", ")
);

impl_find_by_ids_for_db!(
    "postgres",
    sqlx::Postgres,
    sqlx::postgres::PgRow,
    crate::db_pool::DbDriver::Postgres,
    find_by_ids_postgres,
    |len| (1..=len)
        .map(|i| format!("${}", i))
        .collect::<Vec<_>>()
        .join(", ")
);

impl_find_by_ids_for_db!(
    "sqlite",
    sqlx::Sqlite,
    sqlx::sqlite::SqliteRow,
    crate::db_pool::DbDriver::Sqlite,
    find_by_ids_sqlite,
    |len| (0..len).map(|_| "?").collect::<Vec<_>>().join(", ")
);

/// 宏：生成数据库特定版本的 hard_delete_by_id 函数
macro_rules! impl_hard_delete_by_id_for_db {
    (
        $feature:literal,
        $db_type:ty,
        $driver:expr,
        $placeholder:expr,
        $fn_name:ident
    ) => {
        #[cfg(feature = $feature)]
        pub async fn $fn_name<'e, 'c: 'e, M, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, $db_type> + sqlx::Type<$db_type> + Send + Sync,
        ) -> Result<()>
        where
            M: Model,
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            let escaped_table = escape_identifier($driver, M::TABLE);
            let escaped_pk = escape_identifier($driver, M::PK);
            let sql = format!(
                "DELETE FROM {} WHERE {} = {}",
                escaped_table, escaped_pk, $placeholder
            );
            sqlx::query(&sql).bind(id).execute(executor).await?;
            Ok(())
        }
    };
}

// 使用宏生成不同数据库版本的 hard_delete_by_id 函数
impl_hard_delete_by_id_for_db!(
    "mysql",
    sqlx::MySql,
    crate::db_pool::DbDriver::MySql,
    "?",
    hard_delete_by_id_mysql
);

impl_hard_delete_by_id_for_db!(
    "postgres",
    sqlx::Postgres,
    crate::db_pool::DbDriver::Postgres,
    "$1",
    hard_delete_by_id_postgres
);

impl_hard_delete_by_id_for_db!(
    "sqlite",
    sqlx::Sqlite,
    crate::db_pool::DbDriver::Sqlite,
    "?",
    hard_delete_by_id_sqlite
);

/// 宏：生成数据库特定版本的 soft_delete_by_id 函数
macro_rules! impl_soft_delete_by_id_for_db {
    (
        $feature:literal,
        $db_type:ty,
        $driver:expr,
        $placeholder:expr,
        $fn_name:ident
    ) => {
        #[cfg(feature = $feature)]
        pub async fn $fn_name<'e, 'c: 'e, M, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, $db_type> + sqlx::Type<$db_type> + Send + Sync,
        ) -> Result<()>
        where
            M: Model,
            E: sqlx::Executor<'c, Database = $db_type> + Send,
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

            let escaped_table = escape_identifier($driver, M::TABLE);
            let escaped_pk = escape_identifier($driver, M::PK);
            let escaped_field = escape_identifier($driver, soft_delete_field);
            let sql = format!(
                "UPDATE {} SET {} = 1 WHERE {} = {}",
                escaped_table, escaped_field, escaped_pk, $placeholder
            );
            sqlx::query(&sql).bind(id).execute(executor).await?;
            Ok(())
        }
    };
}

// 使用宏生成不同数据库版本的 soft_delete_by_id 函数
impl_soft_delete_by_id_for_db!(
    "mysql",
    sqlx::MySql,
    crate::db_pool::DbDriver::MySql,
    "?",
    soft_delete_by_id_mysql
);

impl_soft_delete_by_id_for_db!(
    "postgres",
    sqlx::Postgres,
    crate::db_pool::DbDriver::Postgres,
    "$1",
    soft_delete_by_id_postgres
);

impl_soft_delete_by_id_for_db!(
    "sqlite",
    sqlx::Sqlite,
    crate::db_pool::DbDriver::Sqlite,
    "?",
    soft_delete_by_id_sqlite
);

/// 宏：生成数据库特定版本的 find_all 函数
macro_rules! impl_find_all_for_db {
    (
        $feature:literal,
        $db_type:ty,
        $row_type:ty,
        $driver:expr,
        $fn_name:ident,
        $apply_binds_fn:ident
    ) => {
        #[cfg(feature = $feature)]
        pub async fn $fn_name<'e, 'c: 'e, M, E>(
            executor: E,
            builder: Option<QueryBuilder>,
        ) -> Result<Vec<M>>
        where
            M: Model + for<'r> sqlx::FromRow<'r, $row_type> + Send + Unpin,
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            // 构建查询构建器
            let escaped_table = escape_identifier($driver, M::TABLE);
            let mut query_builder = builder
                .unwrap_or_else(|| QueryBuilder::new(format!("SELECT * FROM {}", escaped_table)));
            query_builder = query_builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

            // 如果指定了逻辑删除字段，自动添加过滤条件（只查询未删除的记录）
            if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
                query_builder = query_builder.and_eq(soft_delete_field, 0);
            }

            // 限制最多 1000 条
            let mut sql = query_builder.into_sql($driver);
            sql.push_str(" LIMIT 1000");

            let binds = query_builder.binds().to_vec();
            let query = sqlx::query_as::<$db_type, M>(&sql);
            let query = $apply_binds_fn(query, &binds);
            query
                .fetch_all(executor)
                .await
                .map_err(|e| SqlxPlusError::DatabaseError(e))
        }
    };
}

// 使用宏生成不同数据库版本的 find_all 函数
impl_find_all_for_db!(
    "mysql",
    sqlx::MySql,
    sqlx::mysql::MySqlRow,
    crate::db_pool::DbDriver::MySql,
    find_all_mysql,
    apply_binds_to_query_as_mysql
);

impl_find_all_for_db!(
    "postgres",
    sqlx::Postgres,
    sqlx::postgres::PgRow,
    crate::db_pool::DbDriver::Postgres,
    find_all_postgres,
    apply_binds_to_query_as_postgres
);

impl_find_all_for_db!(
    "sqlite",
    sqlx::Sqlite,
    sqlx::sqlite::SqliteRow,
    crate::db_pool::DbDriver::Sqlite,
    find_all_sqlite,
    apply_binds_to_query_as_sqlite
);

/// 宏：生成数据库特定版本的 find_one 函数
macro_rules! impl_find_one_for_db {
    (
        $feature:literal,
        $db_type:ty,
        $row_type:ty,
        $driver:expr,
        $fn_name:ident,
        $apply_binds_fn:ident
    ) => {
        #[cfg(feature = $feature)]
        pub async fn $fn_name<'e, 'c: 'e, M, E>(
            executor: E,
            builder: QueryBuilder,
        ) -> Result<Option<M>>
        where
            M: Model + for<'r> sqlx::FromRow<'r, $row_type> + Send + Unpin,
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            // 构建查询构建器
            let escaped_table = escape_identifier($driver, M::TABLE);
            let mut query_builder = builder;
            query_builder = query_builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

            // 如果指定了逻辑删除字段，自动添加过滤条件（只查询未删除的记录）
            if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
                query_builder = query_builder.and_eq(soft_delete_field, 0);
            }

            // 自动添加 LIMIT 1
            let mut sql = query_builder.into_sql($driver);
            sql.push_str(" LIMIT 1");

            let binds = query_builder.binds().to_vec();
            let query = sqlx::query_as::<$db_type, M>(&sql);
            let query = $apply_binds_fn(query, &binds);
            query
                .fetch_optional(executor)
                .await
                .map_err(|e| SqlxPlusError::DatabaseError(e))
        }
    };
}

// 使用宏生成不同数据库版本的 find_one 函数
impl_find_one_for_db!(
    "mysql",
    sqlx::MySql,
    sqlx::mysql::MySqlRow,
    crate::db_pool::DbDriver::MySql,
    find_one_mysql,
    apply_binds_to_query_as_mysql
);

impl_find_one_for_db!(
    "postgres",
    sqlx::Postgres,
    sqlx::postgres::PgRow,
    crate::db_pool::DbDriver::Postgres,
    find_one_postgres,
    apply_binds_to_query_as_postgres
);

impl_find_one_for_db!(
    "sqlite",
    sqlx::Sqlite,
    sqlx::sqlite::SqliteRow,
    crate::db_pool::DbDriver::Sqlite,
    find_one_sqlite,
    apply_binds_to_query_as_sqlite
);

/// 宏：生成数据库特定版本的 paginate 函数
macro_rules! impl_paginate_for_db {
    (
        $feature:literal,
        $db_type:ty,
        $row_type:ty,
        $driver:expr,
        $fn_name:ident,
        $apply_binds_fn:ident
    ) => {
        #[cfg(feature = $feature)]
        pub async fn $fn_name<'e, 'c: 'e, M, E>(
            executor: E,
            mut builder: QueryBuilder,
            page: u64,
            size: u64,
        ) -> Result<Page<M>>
        where
            M: Model + for<'r> sqlx::FromRow<'r, $row_type> + Send + Unpin,
            E: sqlx::Executor<'c, Database = $db_type> + Send + Clone,
        {
            let offset = (page - 1) * size;
            let escaped_table = escape_identifier($driver, M::TABLE);
            builder = builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

            if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
                builder = builder.and_eq(soft_delete_field, 0);
            }

            let binds = builder.binds().to_vec();
            let count_sql = builder.clone().into_count_sql($driver);
            let mut count_query = sqlx::query(&count_sql);
            for bind in &binds {
                apply_bind_value!(count_query, bind);
            }
            let executor_clone = executor.clone();
            let row = count_query.fetch_one(executor_clone).await?;
            let total = row.get::<i64, _>(0) as u64;

            let data_sql = builder.clone().into_paginated_sql($driver, size, offset);
            let query = sqlx::query_as::<$db_type, M>(&data_sql);
            let query = $apply_binds_fn(query, &binds);
            let items = query
                .fetch_all(executor)
                .await
                .map_err(|e| SqlxPlusError::DatabaseError(e))?;

            Ok(Page::new(items, total, page, size))
        }
    };
}

// 使用宏生成不同数据库版本的 paginate 函数
impl_paginate_for_db!(
    "mysql",
    sqlx::MySql,
    sqlx::mysql::MySqlRow,
    crate::db_pool::DbDriver::MySql,
    paginate_mysql,
    apply_binds_to_query_as_mysql
);

impl_paginate_for_db!(
    "postgres",
    sqlx::Postgres,
    sqlx::postgres::PgRow,
    crate::db_pool::DbDriver::Postgres,
    paginate_postgres,
    apply_binds_to_query_as_postgres
);

impl_paginate_for_db!(
    "sqlite",
    sqlx::Sqlite,
    sqlx::sqlite::SqliteRow,
    crate::db_pool::DbDriver::Sqlite,
    paginate_sqlite,
    apply_binds_to_query_as_sqlite
);

/// 宏：生成数据库特定版本的 count 函数
macro_rules! impl_count_for_db {
    (
        $feature:literal,
        $db_type:ty,
        $driver:expr,
        $fn_name:ident
    ) => {
        #[cfg(feature = $feature)]
        pub async fn $fn_name<'e, 'c: 'e, M, E>(executor: E, builder: QueryBuilder) -> Result<u64>
        where
            M: Model,
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            let escaped_table = escape_identifier($driver, M::TABLE);
            let mut builder = builder;
            builder = builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

            if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
                builder = builder.and_eq(soft_delete_field, 0);
            }

            let binds = builder.binds().to_vec();
            let count_sql = builder.into_count_sql($driver);
            let mut query = sqlx::query(&count_sql);
            for bind in &binds {
                apply_bind_value!(query, bind);
            }
            let row = query.fetch_one(executor).await?;
            Ok(row.get::<i64, _>(0) as u64)
        }
    };
}

// 使用宏生成不同数据库版本的 count 函数
impl_count_for_db!(
    "mysql",
    sqlx::MySql,
    crate::db_pool::DbDriver::MySql,
    count_mysql
);

impl_count_for_db!(
    "postgres",
    sqlx::Postgres,
    crate::db_pool::DbDriver::Postgres,
    count_postgres
);

impl_count_for_db!(
    "sqlite",
    sqlx::Sqlite,
    crate::db_pool::DbDriver::Sqlite,
    count_sqlite
);
