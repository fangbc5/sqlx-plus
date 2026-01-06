use crate::builder::query_builder::QueryBuilder;
use crate::crud::Page;
use crate::error::Result;

/// 主键 ID 类型
pub type Id = i64;

/// Model trait 定义了模型的基本元数据
pub trait Model: Sized {
    /// 表名
    const TABLE: &'static str;
    /// 主键字段名
    const PK: &'static str;
    /// 逻辑删除字段名（可选），如果为 Some，则使用逻辑删除
    const SOFT_DELETE_FIELD: Option<&'static str> = None;
}

/// Crud trait 提供了基本的 CRUD 操作
///
/// 注意：使用此 trait 时，需要确保启用了相应的数据库特性（mysql、postgres、sqlite）
#[async_trait::async_trait]
#[allow(async_fn_in_trait)]
#[cfg_attr(
    all(feature = "mysql", feature = "postgres", feature = "sqlite"),
    doc = "支持所有数据库驱动"
)]
pub trait Crud:
    Model
    + Send
    + Sync
    + Unpin
    + for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>
    + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
    + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>
{
    /// 插入记录
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用 Pool（自动推断为 MySql）
    /// let id = user.insert(pool.mysql_pool()).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// let id = user.insert(tx.as_mysql_executor()).await?;
    /// ```
    async fn insert<'e, 'c: 'e, DB, E>(&self, executor: E) -> Result<Id>
    where
        DB: sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: crate::database_type::DatabaseType<DB = DB> + sqlx::Executor<'c, Database = DB> + Send,
        // PostgreSQL 使用 query_scalar 需要这些约束
        i64: sqlx::Type<DB> + for<'r> sqlx::Decode<'r, DB>,
        usize: sqlx::ColumnIndex<DB::Row>,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        // 注意：只包含三种数据库（MySQL、PostgreSQL、SQLite）都支持的类型
        String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i64: for<'b> sqlx::Encode<'b, DB>,
        i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<i64>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<i32>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<i16>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<f64>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<f32>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<bool>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::DateTime<chrono::Utc>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::DateTime<chrono::Utc>>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::NaiveDateTime: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::NaiveDateTime>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::NaiveDate: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::NaiveDate>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::NaiveTime: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::NaiveTime>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<Vec<u8>>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        serde_json::Value: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<serde_json::Value>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>;

    /// 更新记录（Patch 语义）
    ///
    /// - 非 `Option` 字段：始终参与更新，生成 `SET col = ?` 并绑定当前值。
    /// - `Option` 字段：
    ///   - `Some(v)`：生成 `SET col = ?` 并绑定 `v`；
    ///   - `None`：不生成对应的 `SET` 子句，即**不修改该列**，保留数据库中的原值。
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用 Pool（自动推断为 MySql）
    /// user.update(pool.mysql_pool()).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// user.update(tx.as_mysql_executor()).await?;
    /// ```
    async fn update<'e, 'c: 'e, DB, E>(&self, executor: E) -> Result<()>
    where
        DB: sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: crate::database_type::DatabaseType<DB = DB> + sqlx::Executor<'c, Database = DB> + Send,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<i64>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<i32>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<i16>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<f64>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<f32>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<bool>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::DateTime<chrono::Utc>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::DateTime<chrono::Utc>>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::NaiveDateTime: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::NaiveDateTime>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::NaiveDate: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::NaiveDate>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::NaiveTime: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::NaiveTime>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<Vec<u8>>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        serde_json::Value: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<serde_json::Value>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>;

    /// 更新记录（包含 None 字段的重置，Reset 语义）
    ///
    /// - 非 Option 字段：与 `update` 相同，始终参与更新
    /// - Option 字段：
    ///   - Some(v)：更新为 v
    ///   - None：更新为数据库默认值（等价于 `SET col = DEFAULT`，具体行为由数据库决定）
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用 Pool（自动推断为 MySql）
    /// user.update_with_none(pool.mysql_pool()).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// user.update_with_none(tx.as_mysql_executor()).await?;
    /// ```
    async fn update_with_none<'e, 'c: 'e, DB, E>(&self, executor: E) -> Result<()>
    where
        DB: sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: crate::database_type::DatabaseType<DB = DB> + sqlx::Executor<'c, Database = DB> + Send,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<i64>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<i32>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<i16>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<f64>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<f32>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<bool>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::DateTime<chrono::Utc>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::DateTime<chrono::Utc>>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::NaiveDateTime: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::NaiveDateTime>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::NaiveDate: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::NaiveDate>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        chrono::NaiveTime: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<chrono::NaiveTime>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<Vec<u8>>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        serde_json::Value: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<serde_json::Value>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>;

    /// 根据 ID 查找单条记录
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用 Pool（自动推断为 MySql）
    /// let user = User::find_by_id(pool.mysql_pool(), 1).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// let user = User::find_by_id(tx.as_mysql_executor(), 1).await?;
    /// ```
    async fn find_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: impl for<'q> sqlx::Encode<'q, <E as crate::database_type::DatabaseType>::DB>
            + sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + Send
            + Sync,
    ) -> Result<Option<Self>>
    where
        E: crate::database_type::DatabaseType
            + sqlx::Executor<'c, Database = <E as crate::database_type::DatabaseType>::DB>
            + Send,
        <E as crate::database_type::DatabaseType>::DB:
            sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Arguments<'a>:
            sqlx::IntoArguments<'a, <E as crate::database_type::DatabaseType>::DB>,
        Self: for<'r> sqlx::FromRow<
                'r,
                <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Row,
            > + Send
            + Unpin,
    {
        crate::crud::find_by_id::<<E as crate::database_type::DatabaseType>::DB, Self, E>(
            executor, id,
        )
        .await
    }

    /// 根据多个 ID 查找记录
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用 Pool（自动推断为 MySql）
    /// let users = User::find_by_ids(pool.mysql_pool(), vec![1, 2, 3]).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// let users = User::find_by_ids(tx.as_mysql_executor(), vec![1, 2, 3]).await?;
    /// ```
    async fn find_by_ids<'e, 'c: 'e, I, E>(executor: E, ids: I) -> Result<Vec<Self>>
    where
        E: crate::database_type::DatabaseType
            + sqlx::Executor<'c, Database = <E as crate::database_type::DatabaseType>::DB>
            + Send,
        <E as crate::database_type::DatabaseType>::DB:
            sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Arguments<'a>:
            sqlx::IntoArguments<'a, <E as crate::database_type::DatabaseType>::DB>,
        Self: for<'r> sqlx::FromRow<
                'r,
                <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Row,
            > + Send
            + Unpin,
        I: IntoIterator + Send,
        I::Item: for<'q> sqlx::Encode<'q, <E as crate::database_type::DatabaseType>::DB>
            + sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + Send
            + Sync
            + Clone,
    {
        crate::crud::find_by_ids::<<E as crate::database_type::DatabaseType>::DB, Self, I, E>(
            executor, ids,
        )
        .await
    }

    /// 根据查询构建器查找单条记录
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use sqlxplus::QueryBuilder;
    ///
    /// // 使用 Pool（自动推断为 MySql）
    /// let builder = QueryBuilder::new("SELECT * FROM user").and_eq("id", 1);
    /// let user = User::find_one(pool.mysql_pool(), builder).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// let user = User::find_one(tx.as_mysql_executor(), builder).await?;
    /// ```
    async fn find_one<'e, 'c: 'e, E>(executor: E, builder: QueryBuilder) -> Result<Option<Self>>
    where
        E: crate::database_type::DatabaseType
            + sqlx::Executor<'c, Database = <E as crate::database_type::DatabaseType>::DB>
            + Send,
        <E as crate::database_type::DatabaseType>::DB:
            sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Arguments<'a>:
            sqlx::IntoArguments<'a, <E as crate::database_type::DatabaseType>::DB>,
        Self: for<'r> sqlx::FromRow<
                'r,
                <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Row,
            > + Send
            + Unpin,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        // 注意：只包含三种数据库（MySQL、PostgreSQL、SQLite）都支持的类型
        String: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        i64: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        i32: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        i16: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        f64: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        f32: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        bool: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        Vec<u8>: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        Option<String>: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
    {
        crate::crud::find_one::<<E as crate::database_type::DatabaseType>::DB, Self, E>(
            executor, builder,
        )
        .await
    }

    /// 根据查询构建器查找所有记录
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use sqlxplus::QueryBuilder;
    ///
    /// // 使用 Pool（自动推断为 MySql）- 查询所有记录
    /// let users = User::find_all(pool.mysql_pool(), None).await?;
    ///
    /// // 使用查询构建器
    /// let builder = QueryBuilder::new("SELECT * FROM user").and_eq("status", 1);
    /// let users = User::find_all(pool.mysql_pool(), Some(builder)).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// let users = User::find_all(tx.as_mysql_executor(), None).await?;
    /// ```
    async fn find_all<'e, 'c: 'e, E>(
        executor: E,
        builder: Option<QueryBuilder>,
    ) -> Result<Vec<Self>>
    where
        E: crate::database_type::DatabaseType
            + sqlx::Executor<'c, Database = <E as crate::database_type::DatabaseType>::DB>
            + Send,
        <E as crate::database_type::DatabaseType>::DB:
            sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Arguments<'a>:
            sqlx::IntoArguments<'a, <E as crate::database_type::DatabaseType>::DB>,
        Self: for<'r> sqlx::FromRow<
                'r,
                <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Row,
            > + Send
            + Unpin,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        // 注意：只包含三种数据库（MySQL、PostgreSQL、SQLite）都支持的类型
        String: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        i64: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        i32: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        i16: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        f64: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        f32: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        bool: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        Vec<u8>: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        Option<String>: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
    {
        crate::crud::find_all::<<E as crate::database_type::DatabaseType>::DB, Self, E>(
            executor, builder,
        )
        .await
    }

    /// 统计记录数量
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use sqlxplus::QueryBuilder;
    ///
    /// // 使用 Pool（自动推断为 MySql）
    /// let builder = QueryBuilder::new("SELECT * FROM user");
    /// let count = User::count(pool.mysql_pool(), builder).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// let count = User::count(tx.as_mysql_executor(), builder).await?;
    /// ```
    async fn count<'e, 'c: 'e, E>(executor: E, builder: QueryBuilder) -> Result<u64>
    where
        E: crate::database_type::DatabaseType
            + sqlx::Executor<'c, Database = <E as crate::database_type::DatabaseType>::DB>
            + Send,
        <E as crate::database_type::DatabaseType>::DB:
            sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Arguments<'a>:
            sqlx::IntoArguments<'a, <E as crate::database_type::DatabaseType>::DB>,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        // i64 还需要实现 Decode<DB>（用于从查询结果中读取）
        // usize 需要实现 ColumnIndex<DB::Row>（用于通过索引访问列）
        String: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        i64: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>
            + for<'r> sqlx::Decode<'r, <E as crate::database_type::DatabaseType>::DB>,
        i32: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        i16: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        f64: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        f32: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        bool: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        Vec<u8>: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        Option<String>: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        usize: sqlx::ColumnIndex<
            <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Row,
        >,
    {
        crate::crud::count::<<E as crate::database_type::DatabaseType>::DB, Self, E>(
            executor, builder,
        )
        .await
    }

    /// 分页查询
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use sqlxplus::QueryBuilder;
    ///
    /// // 使用 Pool（自动推断为 MySql）
    /// let builder = QueryBuilder::new("SELECT * FROM user");
    /// let page = User::paginate(pool.mysql_pool(), builder, 1, 10).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// let page = User::paginate(tx.as_mysql_executor(), builder, 1, 10).await?;
    /// ```
    async fn paginate<'e, 'c: 'e, E>(
        executor: E,
        builder: QueryBuilder,
        page: u32,
        size: u32,
    ) -> Result<Page<Self>>
    where
        E: crate::database_type::DatabaseType
            + sqlx::Executor<'c, Database = <E as crate::database_type::DatabaseType>::DB>
            + Send
            + Clone,
        <E as crate::database_type::DatabaseType>::DB:
            sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Arguments<'a>:
            sqlx::IntoArguments<'a, <E as crate::database_type::DatabaseType>::DB>,
        Self: for<'r> sqlx::FromRow<
                'r,
                <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Row,
            > + Send
            + Unpin,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        // i64 还需要实现 Decode<DB>（用于从查询结果中读取）
        // usize 需要实现 ColumnIndex<DB::Row>（用于通过索引访问列）
        String: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        i64: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>
            + for<'r> sqlx::Decode<'r, <E as crate::database_type::DatabaseType>::DB>,
        i32: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        i16: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        f64: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        f32: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        bool: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        Vec<u8>: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        Option<String>: sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + for<'b> sqlx::Encode<'b, <E as crate::database_type::DatabaseType>::DB>,
        usize: sqlx::ColumnIndex<
            <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Row,
        >,
    {
        crate::crud::paginate::<<E as crate::database_type::DatabaseType>::DB, Self, E>(
            executor, builder, page, size,
        )
        .await
    }

    /// 根据 ID 物理删除记录
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用 Pool（自动推断为 MySql）
    /// User::hard_delete_by_id(pool.mysql_pool(), 1).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// User::hard_delete_by_id(tx.as_mysql_executor(), 1).await?;
    /// ```
    async fn hard_delete_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: impl for<'q> sqlx::Encode<'q, <E as crate::database_type::DatabaseType>::DB>
            + sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + Send
            + Sync,
    ) -> Result<()>
    where
        E: crate::database_type::DatabaseType
            + sqlx::Executor<'c, Database = <E as crate::database_type::DatabaseType>::DB>
            + Send,
        <E as crate::database_type::DatabaseType>::DB:
            sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Arguments<'a>:
            sqlx::IntoArguments<'a, <E as crate::database_type::DatabaseType>::DB>,
    {
        crate::crud::hard_delete_by_id::<<E as crate::database_type::DatabaseType>::DB, Self, E>(
            executor, id,
        )
        .await
    }

    /// 根据 ID 逻辑删除记录
    ///
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用 Pool（自动推断为 MySql）
    /// User::soft_delete_by_id(pool.mysql_pool(), 1).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// User::soft_delete_by_id(tx.as_mysql_executor(), 1).await?;
    /// ```
    async fn soft_delete_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: impl for<'q> sqlx::Encode<'q, <E as crate::database_type::DatabaseType>::DB>
            + sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + Send
            + Sync,
    ) -> Result<()>
    where
        E: crate::database_type::DatabaseType
            + sqlx::Executor<'c, Database = <E as crate::database_type::DatabaseType>::DB>
            + Send,
        <E as crate::database_type::DatabaseType>::DB:
            sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Arguments<'a>:
            sqlx::IntoArguments<'a, <E as crate::database_type::DatabaseType>::DB>,
    {
        crate::crud::soft_delete_by_id::<<E as crate::database_type::DatabaseType>::DB, Self, E>(
            executor, id,
        )
        .await
    }

    /// 根据 ID 删除记录
    ///
    /// 如果模型定义了 `SOFT_DELETE_FIELD`，则使用逻辑删除；否则使用物理删除。
    /// 根据传入的 Pool 或 Transaction 自动推断数据库类型，无需显式指定数据库类型参数。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 使用 Pool（自动推断为 MySql）
    /// User::delete_by_id(pool.mysql_pool(), 1).await?;
    ///
    /// // 使用 Transaction（自动推断为 MySql）
    /// User::delete_by_id(tx.as_mysql_executor(), 1).await?;
    /// ```
    async fn delete_by_id<'e, 'c: 'e, E>(
        executor: E,
        id: impl for<'q> sqlx::Encode<'q, <E as crate::database_type::DatabaseType>::DB>
            + sqlx::Type<<E as crate::database_type::DatabaseType>::DB>
            + Send
            + Sync,
    ) -> Result<()>
    where
        E: crate::database_type::DatabaseType
            + sqlx::Executor<'c, Database = <E as crate::database_type::DatabaseType>::DB>
            + Send,
        <E as crate::database_type::DatabaseType>::DB:
            sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> <<E as crate::database_type::DatabaseType>::DB as sqlx::Database>::Arguments<'a>:
            sqlx::IntoArguments<'a, <E as crate::database_type::DatabaseType>::DB>,
    {
        crate::crud::delete_by_id::<<E as crate::database_type::DatabaseType>::DB, Self, E>(
            executor, id,
        )
        .await
    }
}
