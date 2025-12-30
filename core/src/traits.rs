use crate::crud::Page;
use crate::error::Result;
use crate::query_builder::QueryBuilder;

/// 主键 ID 类型
pub type Id = i64;

/// 宏：为 Crud trait 生成 hard_delete_by_id 方法
macro_rules! impl_hard_delete_by_id {
    ($feature:literal, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = $feature)]
        async fn $crud_fn<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, $db_type> + sqlx::Type<$db_type> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            crate::crud::$crud_fn::<Self, E>(executor, id).await
        }
    };
}

/// 宏：为 Crud trait 生成 soft_delete_by_id 方法
macro_rules! impl_soft_delete_by_id {
    ($feature:literal, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = $feature)]
        async fn $crud_fn<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, $db_type> + sqlx::Type<$db_type> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            crate::crud::$crud_fn::<Self, E>(executor, id).await
        }
    };
}

/// 宏：为 Crud trait 生成 delete_by_id 方法
macro_rules! impl_delete_by_id {
    ($feature:literal, $db_type:ty, $crud_fn:ident, $soft_fn:ident, $hard_fn:ident) => {
        #[cfg(feature = $feature)]
        async fn $crud_fn<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, $db_type> + sqlx::Type<$db_type> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            if Self::SOFT_DELETE_FIELD.is_some() {
                Self::$soft_fn(executor, id).await
            } else {
                Self::$hard_fn(executor, id).await
            }
        }
    };
}

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
    /// 插入记录（MySQL）
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    #[cfg(feature = "mysql")]
    async fn insert_mysql<'e, 'c: 'e, E>(&self, executor: E) -> Result<Id>
    where
        E: sqlx::Executor<'c, Database = sqlx::MySql> + Send;

    /// 插入记录（PostgreSQL）
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    #[cfg(feature = "postgres")]
    async fn insert_postgres<'e, 'c: 'e, E>(&self, executor: E) -> Result<Id>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send;

    /// 插入记录（SQLite）
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    #[cfg(feature = "sqlite")]
    async fn insert_sqlite<'e, 'c: 'e, E>(&self, executor: E) -> Result<Id>
    where
        E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send;

    /// 更新记录（Patch 语义）
    ///
    /// - 非 `Option` 字段：始终参与更新，生成 `SET col = ?` 并绑定当前值。
    /// - `Option` 字段：
    ///   - `Some(v)`：生成 `SET col = ?` 并绑定 `v`；
    ///   - `None`：不生成对应的 `SET` 子句，即**不修改该列**，保留数据库中的原值。
    ///
    /// 更新记录（Patch 语义，MySQL）
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    #[cfg(feature = "mysql")]
    async fn update_mysql<'e, 'c: 'e, E>(&self, executor: E) -> Result<()>
    where
        E: sqlx::Executor<'c, Database = sqlx::MySql> + Send;

    /// 更新记录（Patch 语义，PostgreSQL）
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    #[cfg(feature = "postgres")]
    async fn update_postgres<'e, 'c: 'e, E>(&self, executor: E) -> Result<()>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send;

    /// 更新记录（Patch 语义，SQLite）
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    #[cfg(feature = "sqlite")]
    async fn update_sqlite<'e, 'c: 'e, E>(&self, executor: E) -> Result<()>
    where
        E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send;

    /// 更新记录（包含 None 字段的重置，Reset 语义）
    ///
    /// - 非 Option 字段：与 `update` 相同，始终参与更新
    /// - Option 字段：
    ///   - Some(v)：更新为 v
    ///   - None：更新为数据库默认值（等价于 `SET col = DEFAULT`，具体行为由数据库决定）
    ///
    /// 更新记录（Reset 语义，MySQL）
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    #[cfg(feature = "mysql")]
    async fn update_with_none_mysql<'e, 'c: 'e, E>(&self, executor: E) -> Result<()>
    where
        E: sqlx::Executor<'c, Database = sqlx::MySql> + Send;

    /// 更新记录（Reset 语义，PostgreSQL）
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    #[cfg(feature = "postgres")]
    async fn update_with_none_postgres<'e, 'c: 'e, E>(&self, executor: E) -> Result<()>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send;

    /// 更新记录（Reset 语义，SQLite）
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    #[cfg(feature = "sqlite")]
    async fn update_with_none_sqlite<'e, 'c: 'e, E>(&self, executor: E) -> Result<()>
    where
        E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send;

    /// 根据 ID 查找单条记录（泛型版本）
    ///
    /// 使用泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
    /// 调用时需要指定数据库类型参数，例如：`User::find_by_id::<sqlx::MySql, _>(pool, id)`
    ///
    /// # 类型参数
    ///
    /// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
    /// * `E` - 执行器类型，通常可以省略（使用 `_` 让编译器推断）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // MySQL
    /// let user = User::find_by_id::<sqlx::MySql, _>(pool, 1).await?;
    ///
    /// // PostgreSQL
    /// let user = User::find_by_id::<sqlx::Postgres, _>(pool, 1).await?;
    ///
    /// // SQLite
    /// let user = User::find_by_id::<sqlx::Sqlite, _>(pool, 1).await?;
    /// ```
    async fn find_by_id<'e, 'c: 'e, DB, E>(
        executor: E,
        id: impl for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync,
    ) -> Result<Option<Self>>
    where
        DB: sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        Self: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        E: sqlx::Executor<'c, Database = DB> + Send,
    {
        crate::crud::find_by_id::<DB, Self, E>(executor, id).await
    }

    /// 根据多个 ID 查找记录（泛型版本）
    ///
    /// 使用泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
    /// 调用时需要指定数据库类型参数，例如：`User::find_by_ids::<sqlx::MySql, _, _>(pool, vec![1, 2, 3])`
    ///
    /// # 类型参数
    ///
    /// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
    /// * `I` - ID 集合类型，可以是 `Vec<T>` 或其他实现了 `IntoIterator` 的类型
    /// * `E` - 执行器类型，通常可以省略（使用 `_` 让编译器推断）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // MySQL
    /// let users = User::find_by_ids::<sqlx::MySql, _, _>(pool, vec![1, 2, 3]).await?;
    ///
    /// // PostgreSQL
    /// let users = User::find_by_ids::<sqlx::Postgres, _, _>(pool, vec![1, 2, 3]).await?;
    ///
    /// // SQLite
    /// let users = User::find_by_ids::<sqlx::Sqlite, _, _>(pool, vec![1, 2, 3]).await?;
    /// ```
    async fn find_by_ids<'e, 'c: 'e, DB, I, E>(executor: E, ids: I) -> Result<Vec<Self>>
    where
        DB: sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        Self: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        I: IntoIterator + Send,
        I::Item: for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync + Clone,
        E: sqlx::Executor<'c, Database = DB> + Send,
    {
        crate::crud::find_by_ids::<DB, Self, I, E>(executor, ids).await
    }

    /// 根据查询构建器查找单条记录（泛型版本）
    ///
    /// 使用泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
    /// 调用时需要指定数据库类型参数，例如：`User::find_one::<sqlx::MySql, _>(pool, builder)`
    ///
    /// # 类型参数
    ///
    /// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
    /// * `E` - 执行器类型，通常可以省略（使用 `_` 让编译器推断）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use sqlxplus::QueryBuilder;
    ///
    /// // MySQL
    /// let builder = QueryBuilder::new("SELECT * FROM user").and_eq("id", 1);
    /// let user = User::find_one::<sqlx::MySql, _>(pool, builder).await?;
    ///
    /// // PostgreSQL
    /// let builder = QueryBuilder::new("SELECT * FROM \"user\"").and_eq("id", 1);
    /// let user = User::find_one::<sqlx::Postgres, _>(pool, builder).await?;
    ///
    /// // SQLite
    /// let builder = QueryBuilder::new("SELECT * FROM user").and_eq("id", 1);
    /// let user = User::find_one::<sqlx::Sqlite, _>(pool, builder).await?;
    /// ```
    async fn find_one<'e, 'c: 'e, DB, E>(executor: E, builder: QueryBuilder) -> Result<Option<Self>>
    where
        DB: sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        Self: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        E: sqlx::Executor<'c, Database = DB> + Send,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        // 虽然 sqlx 已经为这些类型实现了这些 trait，但在泛型上下文中需要显式声明
        String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    {
        crate::crud::find_one::<DB, Self, E>(executor, builder).await
    }

    /// 根据查询构建器查找所有记录（泛型版本）
    ///
    /// 使用泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
    /// 调用时需要指定数据库类型参数，例如：`User::find_all::<sqlx::MySql, _>(pool, None)`
    ///
    /// # 类型参数
    ///
    /// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
    /// * `E` - 执行器类型，通常可以省略（使用 `_` 让编译器推断）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use sqlxplus::QueryBuilder;
    ///
    /// // MySQL - 查询所有记录
    /// let users = User::find_all::<sqlx::MySql, _>(pool, None).await?;
    ///
    /// // PostgreSQL - 使用查询构建器
    /// let builder = QueryBuilder::new("SELECT * FROM \"user\"").and_eq("status", 1);
    /// let users = User::find_all::<sqlx::Postgres, _>(pool, Some(builder)).await?;
    ///
    /// // SQLite
    /// let users = User::find_all::<sqlx::Sqlite, _>(pool, None).await?;
    /// ```
    async fn find_all<'e, 'c: 'e, DB, E>(
        executor: E,
        builder: Option<QueryBuilder>,
    ) -> Result<Vec<Self>>
    where
        DB: sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        Self: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        E: sqlx::Executor<'c, Database = DB> + Send,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        // 虽然 sqlx 已经为这些类型实现了这些 trait，但在泛型上下文中需要显式声明
        String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    {
        crate::crud::find_all::<DB, Self, E>(executor, builder).await
    }

    /// 统计记录数量（泛型版本）
    ///
    /// 使用泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
    /// 调用时需要指定数据库类型参数，例如：`User::count::<sqlx::MySql, _>(pool, builder)`
    ///
    /// # 类型参数
    ///
    /// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
    /// * `E` - 执行器类型，通常可以省略（使用 `_` 让编译器推断）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use sqlxplus::QueryBuilder;
    ///
    /// // MySQL
    /// let builder = QueryBuilder::new("SELECT * FROM user");
    /// let count = User::count::<sqlx::MySql, _>(pool, builder).await?;
    ///
    /// // PostgreSQL
    /// let builder = QueryBuilder::new("SELECT * FROM \"user\"");
    /// let count = User::count::<sqlx::Postgres, _>(pool, builder).await?;
    ///
    /// // SQLite
    /// let builder = QueryBuilder::new("SELECT * FROM user");
    /// let count = User::count::<sqlx::Sqlite, _>(pool, builder).await?;
    /// ```
    async fn count<'e, 'c: 'e, DB, E>(executor: E, builder: QueryBuilder) -> Result<u64>
    where
        DB: sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: sqlx::Executor<'c, Database = DB> + Send,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        // i64 还需要实现 Decode<DB>（用于从查询结果中读取）
        // usize 需要实现 ColumnIndex<DB::Row>（用于通过索引访问列）
        // 虽然 sqlx 已经为这些类型实现了这些 trait，但在泛型上下文中需要显式声明
        String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB> + for<'r> sqlx::Decode<'r, DB>,
        i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        usize: sqlx::ColumnIndex<DB::Row>,
    {
        crate::crud::count::<DB, Self, E>(executor, builder).await
    }

    /// 分页查询（泛型版本）
    ///
    /// 使用泛型实现，支持所有实现了 `DatabaseInfo` 的数据库类型。
    /// 调用时需要指定数据库类型参数，例如：`User::paginate::<sqlx::MySql, _>(pool, builder, 1, 10)`
    ///
    /// # 类型参数
    ///
    /// * `DB` - 数据库类型（如 `sqlx::MySql`, `sqlx::Postgres`, `sqlx::Sqlite`）
    /// * `E` - 执行器类型，通常可以省略（使用 `_` 让编译器推断），需要实现 `Clone`
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use sqlxplus::QueryBuilder;
    ///
    /// // MySQL
    /// let builder = QueryBuilder::new("SELECT * FROM user");
    /// let page = User::paginate::<sqlx::MySql, _>(pool, builder, 1, 10).await?;
    ///
    /// // PostgreSQL
    /// let builder = QueryBuilder::new("SELECT * FROM \"user\"");
    /// let page = User::paginate::<sqlx::Postgres, _>(pool, builder, 1, 10).await?;
    ///
    /// // SQLite
    /// let builder = QueryBuilder::new("SELECT * FROM user");
    /// let page = User::paginate::<sqlx::Sqlite, _>(pool, builder, 1, 10).await?;
    /// ```
    async fn paginate<'e, 'c: 'e, DB, E>(
        executor: E,
        builder: QueryBuilder,
        page: u64,
        size: u64,
    ) -> Result<Page<Self>>
    where
        DB: sqlx::Database + crate::database_info::DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        Self: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        E: sqlx::Executor<'c, Database = DB> + Send + Clone,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        // i64 还需要实现 Decode<DB>（用于从查询结果中读取）
        // usize 需要实现 ColumnIndex<DB::Row>（用于通过索引访问列）
        // 虽然 sqlx 已经为这些类型实现了这些 trait，但在泛型上下文中需要显式声明
        String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB> + for<'r> sqlx::Decode<'r, DB>,
        i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        usize: sqlx::ColumnIndex<DB::Row>,
    {
        crate::crud::paginate::<DB, Self, E>(executor, builder, page, size).await
    }

    impl_hard_delete_by_id!("mysql", sqlx::MySql, hard_delete_by_id_mysql);
    impl_hard_delete_by_id!("postgres", sqlx::Postgres, hard_delete_by_id_postgres);
    impl_hard_delete_by_id!("sqlite", sqlx::Sqlite, hard_delete_by_id_sqlite);

    impl_soft_delete_by_id!("mysql", sqlx::MySql, soft_delete_by_id_mysql);
    impl_soft_delete_by_id!("postgres", sqlx::Postgres, soft_delete_by_id_postgres);
    impl_soft_delete_by_id!("sqlite", sqlx::Sqlite, soft_delete_by_id_sqlite);

    impl_delete_by_id!(
        "mysql",
        sqlx::MySql,
        delete_by_id_mysql,
        soft_delete_by_id_mysql,
        hard_delete_by_id_mysql
    );
    impl_delete_by_id!(
        "postgres",
        sqlx::Postgres,
        delete_by_id_postgres,
        soft_delete_by_id_postgres,
        hard_delete_by_id_postgres
    );
    impl_delete_by_id!(
        "sqlite",
        sqlx::Sqlite,
        delete_by_id_sqlite,
        soft_delete_by_id_sqlite,
        hard_delete_by_id_sqlite
    );
}
