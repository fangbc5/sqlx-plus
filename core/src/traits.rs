use crate::crud::Page;
use crate::error::{Result, SqlxPlusError};
use crate::query_builder::QueryBuilder;

/// 主键 ID 类型
pub type Id = i64;

/// 宏：为 Crud trait 生成 find_by_id 方法
/// 使用优先级条件编译：mysql > postgres > sqlite
macro_rules! impl_find_by_id {
    (mysql, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = "mysql")]
        async fn find_by_id<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::MySql> + sqlx::Type<sqlx::MySql> + Send + Sync,
        ) -> Result<Option<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::MySql> + Send,
        {
            crate::crud::find_by_id_mysql(executor, id).await
        }
    };
    (postgres, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "postgres", not(feature = "mysql")))]
        async fn find_by_id<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + Sync,
        ) -> Result<Option<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send,
        {
            crate::crud::find_by_id_postgres(executor, id).await
        }
    };
    (sqlite, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))))]
        async fn find_by_id<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Send + Sync,
        ) -> Result<Option<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send,
        {
            crate::crud::find_by_id_sqlite(executor, id).await
        }
    };
}

/// 宏：为 Crud trait 生成 find_by_ids 方法
macro_rules! impl_find_by_ids {
    (mysql, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = "mysql")]
        async fn find_by_ids<'e, 'c: 'e, I, E>(executor: E, ids: I) -> Result<Vec<Self>>
        where
            I: IntoIterator + Send,
            I::Item: for<'q> sqlx::Encode<'q, sqlx::MySql>
                + sqlx::Type<sqlx::MySql>
                + Send
                + Sync
                + Clone,
            E: sqlx::Executor<'c, Database = sqlx::MySql> + Send,
        {
            crate::crud::find_by_ids_mysql(executor, ids).await
        }
    };
    (postgres, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "postgres", not(feature = "mysql")))]
        async fn find_by_ids<'e, 'c: 'e, I, E>(executor: E, ids: I) -> Result<Vec<Self>>
        where
            I: IntoIterator + Send,
            I::Item: for<'q> sqlx::Encode<'q, sqlx::Postgres>
                + sqlx::Type<sqlx::Postgres>
                + Send
                + Sync
                + Clone,
            E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send,
        {
            crate::crud::find_by_ids_postgres(executor, ids).await
        }
    };
    (sqlite, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))))]
        async fn find_by_ids<'e, 'c: 'e, I, E>(executor: E, ids: I) -> Result<Vec<Self>>
        where
            I: IntoIterator + Send,
            I::Item: for<'q> sqlx::Encode<'q, sqlx::Sqlite>
                + sqlx::Type<sqlx::Sqlite>
                + Send
                + Sync
                + Clone,
            E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send,
        {
            crate::crud::find_by_ids_sqlite(executor, ids).await
        }
    };
}

/// 宏：为 Crud trait 生成 find_all 方法
macro_rules! impl_find_all {
    (mysql, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = "mysql")]
        async fn find_all<'e, 'c: 'e, E>(
            executor: E,
            builder: Option<QueryBuilder>,
        ) -> Result<Vec<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::MySql> + Send,
        {
            crate::crud::find_all_mysql(executor, builder).await
        }
    };
    (postgres, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "postgres", not(feature = "mysql")))]
        async fn find_all<E>(executor: E, builder: Option<QueryBuilder>) -> Result<Vec<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send,
        {
            crate::crud::find_all_postgres(executor, builder).await
        }
    };
    (sqlite, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))))]
        async fn find_all<E>(executor: E, builder: Option<QueryBuilder>) -> Result<Vec<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send,
        {
            crate::crud::find_all_sqlite(executor, builder).await
        }
    };
}

/// 宏：为 Crud trait 生成 find_one 方法
macro_rules! impl_find_one {
    (mysql, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = "mysql")]
        async fn find_one<'e, 'c: 'e, E>(executor: E, builder: QueryBuilder) -> Result<Option<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::MySql> + Send,
        {
            crate::crud::find_one_mysql(executor, builder).await
        }
    };
    (postgres, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "postgres", not(feature = "mysql")))]
        async fn find_one<E>(executor: E, builder: QueryBuilder) -> Result<Option<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send,
        {
            crate::crud::find_one_postgres(executor, builder).await
        }
    };
    (sqlite, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))))]
        async fn find_one<E>(executor: E, builder: QueryBuilder) -> Result<Option<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send,
        {
            crate::crud::find_one_sqlite(executor, builder).await
        }
    };
}

/// 宏：为 Crud trait 生成 count 方法
macro_rules! impl_count {
    (mysql, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = "mysql")]
        async fn count<'e, 'c: 'e, E>(executor: E, builder: QueryBuilder) -> Result<u64>
        where
            E: sqlx::Executor<'c, Database = sqlx::MySql> + Send,
        {
            crate::crud::count_mysql::<Self, E>(executor, builder).await
        }
    };
    (postgres, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "postgres", not(feature = "mysql")))]
        async fn count<E>(executor: E, builder: QueryBuilder) -> Result<u64>
        where
            E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send,
        {
            crate::crud::count_postgres::<Self, E>(executor, builder).await
        }
    };
    (sqlite, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))))]
        async fn count<E>(executor: E, builder: QueryBuilder) -> Result<u64>
        where
            E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send,
        {
            crate::crud::count_sqlite::<Self, E>(executor, builder).await
        }
    };
}

/// 宏：为 Crud trait 生成 hard_delete_by_id 方法
macro_rules! impl_hard_delete_by_id {
    (mysql, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = "mysql")]
        async fn hard_delete_by_id<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::MySql> + sqlx::Type<sqlx::MySql> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = sqlx::MySql> + Send,
        {
            crate::crud::hard_delete_by_id_mysql::<Self, E>(executor, id).await
        }
    };
    (postgres, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "postgres", not(feature = "mysql")))]
        async fn hard_delete_by_id<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send,
        {
            crate::crud::hard_delete_by_id_postgres::<Self, E>(executor, id).await
        }
    };
    (sqlite, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))))]
        async fn hard_delete_by_id<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send,
        {
            crate::crud::hard_delete_by_id_sqlite::<Self, E>(executor, id).await
        }
    };
}

/// 宏：为 Crud trait 生成 soft_delete_by_id 方法
macro_rules! impl_soft_delete_by_id {
    (mysql, $db_type:ty, $crud_fn:ident) => {
        #[cfg(feature = "mysql")]
        async fn soft_delete_by_id<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::MySql> + sqlx::Type<sqlx::MySql> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = sqlx::MySql> + Send,
        {
            crate::crud::soft_delete_by_id_mysql::<Self, E>(executor, id).await
        }
    };
    (postgres, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "postgres", not(feature = "mysql")))]
        async fn soft_delete_by_id<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send,
        {
            crate::crud::soft_delete_by_id_postgres::<Self, E>(executor, id).await
        }
    };
    (sqlite, $db_type:ty, $crud_fn:ident) => {
        #[cfg(all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))))]
        async fn soft_delete_by_id<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send,
        {
            crate::crud::soft_delete_by_id_sqlite::<Self, E>(executor, id).await
        }
    };
}

/// 宏：为 Crud trait 生成 delete_by_id 方法
macro_rules! impl_delete_by_id {
    (mysql, $db_type:ty) => {
        #[cfg(feature = "mysql")]
        async fn delete_by_id<'e, 'c: 'e, E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::MySql> + sqlx::Type<sqlx::MySql> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = sqlx::MySql> + Send,
        {
            if Self::SOFT_DELETE_FIELD.is_some() {
                Self::soft_delete_by_id(executor, id).await
            } else {
                Self::hard_delete_by_id(executor, id).await
            }
        }
    };
    (postgres, $db_type:ty) => {
        #[cfg(all(feature = "postgres", not(feature = "mysql")))]
        async fn delete_by_id<E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send,
        {
            if Self::SOFT_DELETE_FIELD.is_some() {
                Self::soft_delete_by_id(executor, id).await
            } else {
                Self::hard_delete_by_id(executor, id).await
            }
        }
    };
    (sqlite, $db_type:ty) => {
        #[cfg(all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))))]
        async fn delete_by_id<E>(
            executor: E,
            id: impl for<'q> sqlx::Encode<'q, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Send + Sync,
        ) -> Result<()>
        where
            E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send,
        {
            if Self::SOFT_DELETE_FIELD.is_some() {
                Self::soft_delete_by_id(executor, id).await
            } else {
                Self::hard_delete_by_id(executor, id).await
            }
        }
    };
}

/// 宏：为 Crud trait 生成 paginate 方法
macro_rules! impl_paginate {
    (mysql, $db_type:ty) => {
        #[cfg(feature = "mysql")]
        async fn paginate<'e, 'c: 'e, E>(
            executor: E,
            builder: QueryBuilder,
            page: u64,
            size: u64,
        ) -> Result<Page<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::MySql> + Send + Clone,
        {
            crate::crud::paginate_mysql::<Self, E>(executor, builder, page, size).await
        }
    };
    (postgres, $db_type:ty) => {
        #[cfg(all(feature = "postgres", not(feature = "mysql")))]
        async fn paginate<E>(
            executor: E,
            builder: QueryBuilder,
            page: u64,
            size: u64,
        ) -> Result<Page<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::Postgres> + Send + Clone,
        {
            crate::crud::paginate_postgres::<Self, E>(executor, builder, page, size).await
        }
    };
    (sqlite, $db_type:ty) => {
        #[cfg(all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))))]
        async fn paginate<E>(
            executor: E,
            builder: QueryBuilder,
            page: u64,
            size: u64,
        ) -> Result<Page<Self>>
        where
            E: sqlx::Executor<'c, Database = sqlx::Sqlite> + Send + Clone,
        {
            crate::crud::paginate_sqlite::<Self, E>(executor, builder, page, size).await
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
    /// 插入记录
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    async fn insert<E>(&self, _executor: E) -> Result<Id>
    where
        E: Send,
    {
        Err(SqlxPlusError::DatabaseError(sqlx::Error::Configuration(
            format!(
                "insert() must be implemented by derive(CRUD) macro for {}",
                std::any::type_name::<Self>()
            )
            .into(),
        )))
    }

    /// 更新记录（Patch 语义）
    ///
    /// - 非 `Option` 字段：始终参与更新，生成 `SET col = ?` 并绑定当前值。
    /// - `Option` 字段：
    ///   - `Some(v)`：生成 `SET col = ?` 并绑定 `v`；
    ///   - `None`：不生成对应的 `SET` 子句，即**不修改该列**，保留数据库中的原值。
    ///
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    async fn update<E>(&self, _executor: E) -> Result<()>
    where
        E: Send,
    {
        Err(SqlxPlusError::DatabaseError(sqlx::Error::Configuration(
            format!(
                "update() must be implemented by derive(CRUD) macro for {}",
                std::any::type_name::<Self>()
            )
            .into(),
        )))
    }

    /// 更新记录（包含 None 字段的重置，Reset 语义）
    ///
    /// - 非 Option 字段：与 `update` 相同，始终参与更新
    /// - Option 字段：
    ///   - Some(v)：更新为 v
    ///   - None：更新为数据库默认值（等价于 `SET col = DEFAULT`，具体行为由数据库决定）
    ///
    /// 注意：此方法必须由 derive(CRUD) 宏生成具体实现
    async fn update_with_none<E>(&self, _executor: E) -> Result<()>
    where
        E: Send,
    {
        Err(SqlxPlusError::DatabaseError(sqlx::Error::Configuration(
            format!(
                "update_with_none() must be implemented by derive(CRUD) macro for {}",
                std::any::type_name::<Self>()
            )
            .into(),
        )))
    }

    impl_find_by_id!(mysql, sqlx::MySql, find_by_id_mysql);
    impl_find_by_id!(postgres, sqlx::Postgres, find_by_id_postgres);
    impl_find_by_id!(sqlite, sqlx::Sqlite, find_by_id_sqlite);

    impl_find_by_ids!(mysql, sqlx::MySql, find_by_ids_mysql);
    impl_find_by_ids!(postgres, sqlx::Postgres, find_by_ids_postgres);
    impl_find_by_ids!(sqlite, sqlx::Sqlite, find_by_ids_sqlite);

    impl_find_all!(mysql, sqlx::MySql, find_all_mysql);
    impl_find_all!(postgres, sqlx::Postgres, find_all_postgres);
    impl_find_all!(sqlite, sqlx::Sqlite, find_all_sqlite);

    impl_find_one!(mysql, sqlx::MySql, find_one_mysql);
    impl_find_one!(postgres, sqlx::Postgres, find_one_postgres);
    impl_find_one!(sqlite, sqlx::Sqlite, find_one_sqlite);

    impl_count!(mysql, sqlx::MySql, count_mysql);
    impl_count!(postgres, sqlx::Postgres, count_postgres);
    impl_count!(sqlite, sqlx::Sqlite, count_sqlite);

    impl_hard_delete_by_id!(mysql, sqlx::MySql, hard_delete_by_id_mysql);
    impl_hard_delete_by_id!(postgres, sqlx::Postgres, hard_delete_by_id_postgres);
    impl_hard_delete_by_id!(sqlite, sqlx::Sqlite, hard_delete_by_id_sqlite);

    impl_soft_delete_by_id!(mysql, sqlx::MySql, soft_delete_by_id_mysql);
    impl_soft_delete_by_id!(postgres, sqlx::Postgres, soft_delete_by_id_postgres);
    impl_soft_delete_by_id!(sqlite, sqlx::Sqlite, soft_delete_by_id_sqlite);

    impl_delete_by_id!(mysql, sqlx::MySql);
    impl_delete_by_id!(postgres, sqlx::Postgres);
    impl_delete_by_id!(sqlite, sqlx::Sqlite);

    impl_paginate!(mysql, sqlx::MySql);
    impl_paginate!(postgres, sqlx::Postgres);
    impl_paginate!(sqlite, sqlx::Sqlite);
}
