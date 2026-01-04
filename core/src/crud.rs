use crate::builder::query_builder::{BindValue, QueryBuilder};
use crate::database_info::DatabaseInfo;
use crate::error::{Result, SqlxPlusError};
use crate::traits::Model;
use sqlx::{Database, Row};

/// 主键 ID 类型
pub type Id = i64;

/// 辅助函数：将单个绑定值应用到查询中
/// 这是一个通用的绑定逻辑，通过宏来应用到不同的查询类型
#[macro_export]
macro_rules! apply_bind_value {
    ($query:expr, $bind:expr) => {
        match $bind {
            $crate::builder::query_builder::BindValue::String(s) => {
                $query = $query.bind(s);
            }
            $crate::builder::query_builder::BindValue::Int64(i) => {
                $query = $query.bind(i);
            }
            $crate::builder::query_builder::BindValue::Int32(i) => {
                $query = $query.bind(i);
            }
            $crate::builder::query_builder::BindValue::Int16(i) => {
                $query = $query.bind(i);
            }
            // i8, u64, u32, u16, u8 在 PostgreSQL 中不支持，但保留在 BindValue 中用于 QueryBuilder
            // 这些类型在 CRUD 操作中会通过类型转换处理
            // 注意：当 $bind 是引用时，需要使用 ref 模式或解引用
            $crate::builder::query_builder::BindValue::Int8(ref i) => {
                // 转换为 i16（三种数据库都支持的最小整数类型）
                $query = $query.bind(*i as i16);
            }
            $crate::builder::query_builder::BindValue::UInt64(ref i) => {
                // 转换为 i64（注意：可能溢出，但这是跨数据库兼容的折中方案）
                $query = $query.bind(*i as i64);
            }
            $crate::builder::query_builder::BindValue::UInt32(ref i) => {
                // 转换为 i64
                $query = $query.bind(*i as i64);
            }
            $crate::builder::query_builder::BindValue::UInt16(ref i) => {
                // 转换为 i32
                $query = $query.bind(*i as i32);
            }
            $crate::builder::query_builder::BindValue::UInt8(ref i) => {
                // 转换为 i16
                $query = $query.bind(*i as i16);
            }
            $crate::builder::query_builder::BindValue::Float64(f) => {
                $query = $query.bind(f);
            }
            $crate::builder::query_builder::BindValue::Float32(f) => {
                $query = $query.bind(f);
            }
            $crate::builder::query_builder::BindValue::Bool(b) => {
                $query = $query.bind(b);
            }
            $crate::builder::query_builder::BindValue::Bytes(b) => {
                $query = $query.bind(b);
            }
            $crate::builder::query_builder::BindValue::Null => {
                $query = $query.bind(Option::<String>::None);
            }
        }
    };
}

/// 泛型版本的绑定辅助函数：将绑定值应用到查询中（用于 query）
///
/// 这是统一的泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
/// 用于处理 `sqlx::query` 类型的查询对象。
///
/// # 类型参数
///
/// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
///
/// # 参数
///
/// * `query` - sqlx 的 `Query` 查询对象
/// * `binds` - 绑定值数组
///
/// # 返回值
///
/// 应用了绑定值的查询对象
fn apply_binds_to_query_generic<'q, DB>(
    mut query: sqlx::query::Query<'q, DB, DB::Arguments<'q>>,
    binds: &'q [BindValue],
) -> sqlx::query::Query<'q, DB, DB::Arguments<'q>>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    // 基本类型必须实现 Type<DB> 和 Encode<DB>（这些对于 sqlx 支持的所有数据库都自动满足）
    // 注意：只包含三种数据库（MySQL、PostgreSQL、SQLite）都支持的类型
    String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
{
    for bind in binds {
        crate::apply_bind_value!(query, bind);
    }
    query
}

/// 泛型版本的绑定辅助函数：将绑定值应用到查询中（用于 query_as）
///
/// 这是统一的泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
/// 使用此函数可以避免为每个数据库类型创建单独的绑定函数。
///
/// # 类型参数
///
/// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
/// * `M` - 模型类型，必须实现对应数据库的 `FromRow`
///
/// # 参数
///
/// * `query` - sqlx 的 `QueryAs` 查询对象
/// * `binds` - 绑定值数组
///
/// # 返回值
///
/// 应用了绑定值的查询对象
fn apply_binds_to_query_as_generic<'q, DB, M>(
    mut query: sqlx::query::QueryAs<'q, DB, M, DB::Arguments<'q>>,
    binds: &'q [BindValue],
) -> sqlx::query::QueryAs<'q, DB, M, DB::Arguments<'q>>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: for<'r> sqlx::FromRow<'r, DB::Row>,
    // 基本类型必须实现 Type<DB> 和 Encode<DB>（这些对于 sqlx 支持的所有数据库都自动满足）
    // 注意：只包含三种数据库（MySQL、PostgreSQL、SQLite）都支持的类型
    String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
{
    for bind in binds {
        crate::apply_bind_value!(query, bind);
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

/// 根据多个 ID 查找记录（泛型版本）
///
/// 这是统一的泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
///
/// # 类型参数
///
/// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
/// * `M` - 模型类型，必须实现 `Model` trait 和对应数据库的 `FromRow`
/// * `I` - ID 集合类型，可以是 `Vec<T>` 或其他实现了 `IntoIterator` 的类型
/// * `E` - 执行器类型，可以是连接池或事务
///
/// # 参数
///
/// * `executor` - 数据库执行器（连接池或事务）
/// * `ids` - 主键 ID 集合
///
/// # 返回值
///
/// 返回找到的所有记录，如果没有找到任何记录，返回空向量
///
/// # 示例
///
/// ```rust,ignore
/// use sqlxplus::{DatabaseInfo, crud};
///
/// // MySQL
/// let users = crud::find_by_ids::<sqlx::MySql, User, _, _>(pool, vec![1, 2, 3]).await?;
///
/// // PostgreSQL
/// let users = crud::find_by_ids::<sqlx::Postgres, User, _, _>(pool, vec![1, 2, 3]).await?;
///
/// // SQLite
/// let users = crud::find_by_ids::<sqlx::Sqlite, User, _, _>(pool, vec![1, 2, 3]).await?;
/// ```
pub async fn find_by_ids<'e, 'c: 'e, DB, M, I, E>(executor: E, ids: I) -> Result<Vec<M>>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: Model + for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
    I: IntoIterator + Send,
    I::Item: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync + Clone,
    E: sqlx::Executor<'c, Database = DB> + Send,
{
    let ids_vec: Vec<_> = ids.into_iter().collect();
    if ids_vec.is_empty() {
        return Ok(Vec::new());
    }

    // 使用 DatabaseInfo trait 获取数据库特定信息
    let escaped_table = DB::escape_identifier(M::TABLE);
    let escaped_pk = DB::escape_identifier(M::PK);

    // 为每个 ID 生成占位符
    let placeholders: Vec<String> = (0..ids_vec.len()).map(|i| DB::placeholder(i)).collect();
    let placeholders_str = placeholders.join(", ");

    let mut sql_str = format!(
        "SELECT * FROM {} WHERE {} IN ({})",
        escaped_table, escaped_pk, placeholders_str
    );

    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        let escaped_field = DB::escape_identifier(soft_delete_field);
        sql_str.push_str(&format!(" AND {} = 0", escaped_field));
    }

    // 执行查询
    let mut query = sqlx::query_as::<DB, M>(&sql_str);
    for id in &ids_vec {
        query = query.bind(id.clone());
    }
    query
        .fetch_all(executor)
        .await
        .map_err(|e| SqlxPlusError::DatabaseError(e))
}

// 注意：find_by_ids_mysql, find_by_ids_postgres, find_by_ids_sqlite 等兼容层函数已移除
// 现在直接使用泛型版本的 find_by_ids<DB, M, I, E>
// trait 中的方法直接调用泛型版本，不再需要这些中间函数

/// 根据查询构建器查找单条记录（泛型版本）
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
/// * `builder` - 查询构建器
///
/// # 返回值
///
/// 如果找到记录，返回 `Ok(Some(M))`；如果未找到，返回 `Ok(None)`
///
/// # 示例
///
/// ```rust,ignore
/// use sqlxplus::{DatabaseInfo, crud, QueryBuilder};
///
/// // MySQL
/// let builder = QueryBuilder::new("SELECT * FROM user").and_eq("id", 1);
/// let user = crud::find_one::<sqlx::MySql, User, _>(pool, builder).await?;
///
/// // PostgreSQL
/// let builder = QueryBuilder::new("SELECT * FROM \"user\"").and_eq("id", 1);
/// let user = crud::find_one::<sqlx::Postgres, User, _>(pool, builder).await?;
///
/// // SQLite
/// let builder = QueryBuilder::new("SELECT * FROM user").and_eq("id", 1);
/// let user = crud::find_one::<sqlx::Sqlite, User, _>(pool, builder).await?;
/// ```
pub async fn find_one<'e, 'c: 'e, DB, M, E>(executor: E, builder: QueryBuilder) -> Result<Option<M>>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: Model + for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
    E: sqlx::Executor<'c, Database = DB> + Send,
    // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
    // 虽然 sqlx 已经为这些类型实现了这些 trait，但在泛型上下文中需要显式声明
    // 注意：只包含三种数据库（MySQL、PostgreSQL、SQLite）都支持的类型
    String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
{
    // 使用 DatabaseInfo trait 获取数据库特定信息
    let driver = DB::get_driver();
    let escaped_table = DB::escape_identifier(M::TABLE);

    // 构建查询构建器
    let mut query_builder = builder;
    query_builder = query_builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

    // 如果指定了逻辑删除字段，自动添加过滤条件（只查询未删除的记录）
    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        query_builder = query_builder.and_eq(soft_delete_field, 0);
    }

    // 自动添加 LIMIT 1
    let mut sql = query_builder.into_sql(driver);
    sql.push_str(" LIMIT 1");

    let binds = query_builder.binds().to_vec();
    let query = sqlx::query_as::<DB, M>(&sql);
    let query = apply_binds_to_query_as_generic(query, &binds);

    query
        .fetch_optional(executor)
        .await
        .map_err(|e| SqlxPlusError::DatabaseError(e))
}

// 注意：find_one_mysql, find_one_postgres, find_one_sqlite 等兼容层函数已移除
// 现在直接使用泛型版本的 find_one<DB, M, E>
// trait 中的方法直接调用泛型版本，不再需要这些中间函数

/// 根据查询构建器查找所有记录（泛型版本）
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
/// * `builder` - 查询构建器（可选），如果为 `None`，则查询所有记录
///
/// # 返回值
///
/// 返回找到的所有记录，最多 1000 条
///
/// # 示例
///
/// ```rust,ignore
/// use sqlxplus::{DatabaseInfo, crud, QueryBuilder};
///
/// // MySQL - 查询所有记录
/// let users = crud::find_all::<sqlx::MySql, User, _>(pool, None).await?;
///
/// // PostgreSQL - 使用查询构建器
/// let builder = QueryBuilder::new("SELECT * FROM \"user\"").and_eq("status", 1);
/// let users = crud::find_all::<sqlx::Postgres, User, _>(pool, Some(builder)).await?;
///
/// // SQLite
/// let users = crud::find_all::<sqlx::Sqlite, User, _>(pool, None).await?;
/// ```
pub async fn find_all<'e, 'c: 'e, DB, M, E>(
    executor: E,
    builder: Option<QueryBuilder>,
) -> Result<Vec<M>>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: Model + for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
    E: sqlx::Executor<'c, Database = DB> + Send,
    // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
    // 虽然 sqlx 已经为这些类型实现了这些 trait，但在泛型上下文中需要显式声明
    // 注意：只包含三种数据库（MySQL、PostgreSQL、SQLite）都支持的类型
    String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
{
    // 使用 DatabaseInfo trait 获取数据库特定信息
    let driver = DB::get_driver();
    let escaped_table = DB::escape_identifier(M::TABLE);

    // 构建查询构建器
    let mut query_builder =
        builder.unwrap_or_else(|| QueryBuilder::new(format!("SELECT * FROM {}", escaped_table)));
    query_builder = query_builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

    // 如果指定了逻辑删除字段，自动添加过滤条件（只查询未删除的记录）
    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        query_builder = query_builder.and_eq(soft_delete_field, 0);
    }

    // 限制最多 1000 条
    let mut sql = query_builder.into_sql(driver);
    sql.push_str(" LIMIT 1000");

    let binds = query_builder.binds().to_vec();
    let query = sqlx::query_as::<DB, M>(&sql);
    let query = apply_binds_to_query_as_generic(query, &binds);

    query
        .fetch_all(executor)
        .await
        .map_err(|e| SqlxPlusError::DatabaseError(e))
}

// 注意：find_all_mysql, find_all_postgres, find_all_sqlite 等兼容层函数已移除
// 现在直接使用泛型版本的 find_all<DB, M, E>
// trait 中的方法直接调用泛型版本，不再需要这些中间函数

/// 统计记录数量（泛型版本）
///
/// 这是统一的泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
///
/// # 类型参数
///
/// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
/// * `M` - 模型类型，必须实现 `Model` trait
/// * `E` - 执行器类型，可以是连接池或事务
///
/// # 参数
///
/// * `executor` - 数据库执行器（连接池或事务）
/// * `builder` - 查询构建器
///
/// # 返回值
///
/// 返回符合条件的记录数量
///
/// # 示例
///
/// ```rust,ignore
/// use sqlxplus::{DatabaseInfo, crud, QueryBuilder};
///
/// // MySQL
/// let builder = QueryBuilder::new("SELECT * FROM user");
/// let count = crud::count::<sqlx::MySql, User, _>(pool, builder).await?;
///
/// // PostgreSQL
/// let builder = QueryBuilder::new("SELECT * FROM \"user\"");
/// let count = crud::count::<sqlx::Postgres, User, _>(pool, builder).await?;
///
/// // SQLite
/// let builder = QueryBuilder::new("SELECT * FROM user");
/// let count = crud::count::<sqlx::Sqlite, User, _>(pool, builder).await?;
/// ```
pub async fn count<'e, 'c: 'e, DB, M, E>(executor: E, builder: QueryBuilder) -> Result<u64>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: Model,
    E: sqlx::Executor<'c, Database = DB> + Send,
    // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
    // i64 还需要实现 Decode<DB>（用于从查询结果中读取）
    // usize 需要实现 ColumnIndex<DB::Row>（用于通过索引访问列）
    // 虽然 sqlx 已经为这些类型实现了这些 trait，但在泛型上下文中需要显式声明
    // 注意：只包含三种数据库（MySQL、PostgreSQL、SQLite）都支持的类型
    String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB> + for<'r> sqlx::Decode<'r, DB>,
    i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    usize: sqlx::ColumnIndex<DB::Row>,
{
    // 使用 DatabaseInfo trait 获取数据库特定信息
    let driver = DB::get_driver();
    let escaped_table = DB::escape_identifier(M::TABLE);

    let mut query_builder = builder;
    query_builder = query_builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        query_builder = query_builder.and_eq(soft_delete_field, 0);
    }

    let binds = query_builder.binds().to_vec();
    let count_sql = query_builder.into_count_sql(driver);
    let query = sqlx::query::<DB>(&count_sql);
    let query = apply_binds_to_query_generic(query, &binds);

    let row = query.fetch_one(executor).await?;
    // 使用 get 方法，明确指定索引类型为 usize
    let count_value: i64 = row.get(0usize);
    Ok(count_value as u64)
}

// 注意：count_mysql, count_postgres, count_sqlite 等兼容层函数已移除
// 现在直接使用泛型版本的 count<DB, M, E>
// trait 中的方法直接调用泛型版本，不再需要这些中间函数

/// 分页查询（泛型版本）
///
/// 这是统一的泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
///
/// # 类型参数
///
/// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
/// * `M` - 模型类型，必须实现 `Model` trait 和对应数据库的 `FromRow`
/// * `E` - 执行器类型，可以是连接池或事务（需要实现 `Clone`）
///
/// # 参数
///
/// * `executor` - 数据库执行器（连接池或事务）
/// * `builder` - 查询构建器
/// * `page` - 页码（从 1 开始）
/// * `size` - 每页大小
///
/// # 返回值
///
/// 返回分页结果，包含数据列表、总数、页码等信息
///
/// # 示例
///
/// ```rust,ignore
/// use sqlxplus::{DatabaseInfo, crud, QueryBuilder};
///
/// // MySQL
/// let builder = QueryBuilder::new("SELECT * FROM user");
/// let page = crud::paginate::<sqlx::MySql, User, _>(pool, builder, 1, 10).await?;
///
/// // PostgreSQL
/// let builder = QueryBuilder::new("SELECT * FROM \"user\"");
/// let page = crud::paginate::<sqlx::Postgres, User, _>(pool, builder, 1, 10).await?;
///
/// // SQLite
/// let builder = QueryBuilder::new("SELECT * FROM user");
/// let page = crud::paginate::<sqlx::Sqlite, User, _>(pool, builder, 1, 10).await?;
/// ```
pub async fn paginate<'e, 'c: 'e, DB, M, E>(
    executor: E,
    mut builder: QueryBuilder,
    page: u64,
    size: u64,
) -> Result<Page<M>>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: Model + for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
    E: sqlx::Executor<'c, Database = DB> + Send + Clone,
    // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
    // i64 还需要实现 Decode<DB>（用于从查询结果中读取）
    // usize 需要实现 ColumnIndex<DB::Row>（用于通过索引访问列）
    // 虽然 sqlx 已经为这些类型实现了这些 trait，但在泛型上下文中需要显式声明
    // 注意：只包含三种数据库（MySQL、PostgreSQL、SQLite）都支持的类型
    String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB> + for<'r> sqlx::Decode<'r, DB>,
    i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    usize: sqlx::ColumnIndex<DB::Row>,
{
    let offset = (page - 1) * size;
    let driver = DB::get_driver();
    let escaped_table = DB::escape_identifier(M::TABLE);

    builder = builder.with_base_sql(format!("SELECT * FROM {}", escaped_table));

    if let Some(soft_delete_field) = M::SOFT_DELETE_FIELD {
        builder = builder.and_eq(soft_delete_field, 0);
    }

    let binds = builder.binds().to_vec();

    // 执行 count 查询获取总数
    let count_sql = builder.clone().into_count_sql(driver);
    let count_query = sqlx::query::<DB>(&count_sql);
    let count_query = apply_binds_to_query_generic(count_query, &binds);
    let executor_clone = executor.clone();
    let row = count_query.fetch_one(executor_clone).await?;
    let total: i64 = row.get(0usize);
    let total = total as u64;

    // 执行分页查询获取数据
    let data_sql = builder.clone().into_paginated_sql(driver, size, offset);
    let query = sqlx::query_as::<DB, M>(&data_sql);
    let query = apply_binds_to_query_as_generic(query, &binds);
    let items = query
        .fetch_all(executor)
        .await
        .map_err(|e| SqlxPlusError::DatabaseError(e))?;

    Ok(Page::new(items, total, page, size))
}

// 注意：paginate_mysql, paginate_postgres, paginate_sqlite 等兼容层函数已移除
// 现在直接使用泛型版本的 paginate<DB, M, E>
// trait 中的方法直接调用泛型版本，不再需要这些中间函数

/// 根据 ID 物理删除记录（泛型版本）
///
/// 这是统一的泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
///
/// # 类型参数
///
/// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
/// * `M` - 模型类型，必须实现 `Model` trait
/// * `E` - 执行器类型，可以是连接池或事务
///
/// # 参数
///
/// * `executor` - 数据库执行器（连接池或事务）
/// * `id` - 主键 ID 值
///
/// # 返回值
///
/// 删除成功返回 `Ok(())`
///
/// # 示例
///
/// ```rust,ignore
/// use sqlxplus::{DatabaseInfo, crud};
///
/// // MySQL
/// crud::hard_delete_by_id::<sqlx::MySql, User, _>(pool, 1).await?;
///
/// // PostgreSQL
/// crud::hard_delete_by_id::<sqlx::Postgres, User, _>(pool, 1).await?;
///
/// // SQLite
/// crud::hard_delete_by_id::<sqlx::Sqlite, User, _>(pool, 1).await?;
/// ```
pub async fn hard_delete_by_id<'e, 'c: 'e, DB, M, E>(
    executor: E,
    id: impl for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync,
) -> Result<()>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: Model,
    E: sqlx::Executor<'c, Database = DB> + Send,
{
    let escaped_table = DB::escape_identifier(M::TABLE);
    let escaped_pk = DB::escape_identifier(M::PK);
    let placeholder = DB::placeholder(0);
    let sql = format!(
        "DELETE FROM {} WHERE {} = {}",
        escaped_table, escaped_pk, placeholder
    );
    sqlx::query(&sql).bind(id).execute(executor).await?;
    Ok(())
}

/// 根据 ID 逻辑删除记录（泛型版本）
///
/// 这是统一的泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
/// 逻辑删除会将 `SOFT_DELETE_FIELD` 字段设置为 1。
///
/// # 类型参数
///
/// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
/// * `M` - 模型类型，必须实现 `Model` trait 且定义了 `SOFT_DELETE_FIELD`
/// * `E` - 执行器类型，可以是连接池或事务
///
/// # 参数
///
/// * `executor` - 数据库执行器（连接池或事务）
/// * `id` - 主键 ID 值
///
/// # 返回值
///
/// 删除成功返回 `Ok(())`；如果模型未定义 `SOFT_DELETE_FIELD`，返回错误
///
/// # 示例
///
/// ```rust,ignore
/// use sqlxplus::{DatabaseInfo, crud};
///
/// // MySQL
/// crud::soft_delete_by_id::<sqlx::MySql, User, _>(pool, 1).await?;
///
/// // PostgreSQL
/// crud::soft_delete_by_id::<sqlx::Postgres, User, _>(pool, 1).await?;
///
/// // SQLite
/// crud::soft_delete_by_id::<sqlx::Sqlite, User, _>(pool, 1).await?;
/// ```
pub async fn soft_delete_by_id<'e, 'c: 'e, DB, M, E>(
    executor: E,
    id: impl for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync,
) -> Result<()>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: Model,
    E: sqlx::Executor<'c, Database = DB> + Send,
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

    let escaped_table = DB::escape_identifier(M::TABLE);
    let escaped_pk = DB::escape_identifier(M::PK);
    let escaped_field = DB::escape_identifier(soft_delete_field);
    let placeholder = DB::placeholder(0);
    let sql = format!(
        "UPDATE {} SET {} = 1 WHERE {} = {}",
        escaped_table, escaped_field, escaped_pk, placeholder
    );
    sqlx::query(&sql).bind(id).execute(executor).await?;
    Ok(())
}

/// 根据 ID 删除记录（泛型版本）
///
/// 这是统一的泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
/// 如果模型定义了 `SOFT_DELETE_FIELD`，则使用逻辑删除；否则使用物理删除。
///
/// # 类型参数
///
/// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
/// * `M` - 模型类型，必须实现 `Model` trait
/// * `E` - 执行器类型，可以是连接池或事务
///
/// # 参数
///
/// * `executor` - 数据库执行器（连接池或事务）
/// * `id` - 主键 ID 值
///
/// # 返回值
///
/// 删除成功返回 `Ok(())`
///
/// # 示例
///
/// ```rust,ignore
/// use sqlxplus::{DatabaseInfo, crud};
///
/// // MySQL
/// crud::delete_by_id::<sqlx::MySql, User, _>(pool, 1).await?;
///
/// // PostgreSQL
/// crud::delete_by_id::<sqlx::Postgres, User, _>(pool, 1).await?;
///
/// // SQLite
/// crud::delete_by_id::<sqlx::Sqlite, User, _>(pool, 1).await?;
/// ```
pub async fn delete_by_id<'e, 'c: 'e, DB, M, E>(
    executor: E,
    id: impl for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync,
) -> Result<()>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: Model,
    E: sqlx::Executor<'c, Database = DB> + Send,
{
    if M::SOFT_DELETE_FIELD.is_some() {
        soft_delete_by_id::<DB, M, E>(executor, id).await
    } else {
        hard_delete_by_id::<DB, M, E>(executor, id).await
    }
}
