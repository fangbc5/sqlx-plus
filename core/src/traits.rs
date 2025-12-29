use crate::crud::Page;
use crate::error::Result;
use crate::query_builder::QueryBuilder;

/// 主键 ID 类型
pub type Id = i64;

/// 宏：为 Crud trait 生成 find_by_id 方法
macro_rules! impl_find_by_id {
    ($feature:literal, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = $feature)]
        async fn $crud_fn<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, $db_type> + sqlx::Type<$db_type> + Send + Sync,
        ) -> Result<Option<Self>>
        where
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            crate::crud::$crud_fn(executor, id).await
        }
    };
}

/// 宏：为 Crud trait 生成 find_by_ids 方法
macro_rules! impl_find_by_ids {
    ($feature:literal, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = $feature)]
        async fn $crud_fn<'e, 'c: 'e, I, E>(executor: E, ids: I) -> Result<Vec<Self>>
        where
            I: IntoIterator + Send,
            I::Item:
                for<'q> sqlx::Encode<'q, $db_type> + sqlx::Type<$db_type> + Send + Sync + Clone,
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            crate::crud::$crud_fn(executor, ids).await
        }
    };
}

/// 宏：为 Crud trait 生成 find_all 方法
macro_rules! impl_find_all {
    ($feature:literal, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = $feature)]
        async fn $crud_fn<'e, 'c: 'e, E>(
            executor: E,
            builder: Option<QueryBuilder>,
        ) -> Result<Vec<Self>>
        where
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            crate::crud::$crud_fn(executor, builder).await
        }
    };
}

/// 宏：为 Crud trait 生成 find_one 方法
macro_rules! impl_find_one {
    ($feature:literal, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = $feature)]
        async fn $crud_fn<'e, 'c: 'e, E>(executor: E, builder: QueryBuilder) -> Result<Option<Self>>
        where
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            crate::crud::$crud_fn(executor, builder).await
        }
    };
}

/// 宏：为 Crud trait 生成 count 方法
macro_rules! impl_count {
    ($feature:literal, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = $feature)]
        async fn $crud_fn<'e, 'c: 'e, E>(executor: E, builder: QueryBuilder) -> Result<u64>
        where
            E: sqlx::Executor<'c, Database = $db_type> + Send,
        {
            crate::crud::$crud_fn::<Self, E>(executor, builder).await
        }
    };
}

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

/// 宏：为 Crud trait 生成 paginate 方法
macro_rules! impl_paginate {
    ($feature:literal, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = $feature)]
        async fn $crud_fn<'e, 'c: 'e, E>(
            executor: E,
            builder: QueryBuilder,
            page: u64,
            size: u64,
        ) -> Result<Page<Self>>
        where
            E: sqlx::Executor<'c, Database = $db_type> + Send + Clone,
        {
            crate::crud::$crud_fn::<Self, E>(executor, builder, page, size).await
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

    impl_find_by_id!("mysql", sqlx::MySql, find_by_id_mysql);
    impl_find_by_id!("postgres", sqlx::Postgres, find_by_id_postgres);
    impl_find_by_id!("sqlite", sqlx::Sqlite, find_by_id_sqlite);

    impl_find_by_ids!("mysql", sqlx::MySql, find_by_ids_mysql);
    impl_find_by_ids!("postgres", sqlx::Postgres, find_by_ids_postgres);
    impl_find_by_ids!("sqlite", sqlx::Sqlite, find_by_ids_sqlite);

    impl_find_all!("mysql", sqlx::MySql, find_all_mysql);
    impl_find_all!("postgres", sqlx::Postgres, find_all_postgres);
    impl_find_all!("sqlite", sqlx::Sqlite, find_all_sqlite);

    impl_find_one!("mysql", sqlx::MySql, find_one_mysql);
    impl_find_one!("postgres", sqlx::Postgres, find_one_postgres);
    impl_find_one!("sqlite", sqlx::Sqlite, find_one_sqlite);

    impl_count!("mysql", sqlx::MySql, count_mysql);
    impl_count!("postgres", sqlx::Postgres, count_postgres);
    impl_count!("sqlite", sqlx::Sqlite, count_sqlite);

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

    impl_paginate!("mysql", sqlx::MySql, paginate_mysql);
    impl_paginate!("postgres", sqlx::Postgres, paginate_postgres);
    impl_paginate!("sqlite", sqlx::Sqlite, paginate_sqlite);
}
