use crate::crud::{Id, Page};
use crate::db_pool::Result;
use crate::executor::DbExecutor;
use crate::query_builder::QueryBuilder;

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
    /// 根据 ID 查找记录
    async fn find_by_id<E>(
        executor: &mut E,
        id: impl for<'q> sqlx::Encode<'q, sqlx::MySql>
            + for<'q> sqlx::Encode<'q, sqlx::Postgres>
            + for<'q> sqlx::Encode<'q, sqlx::Sqlite>
            + sqlx::Type<sqlx::MySql>
            + sqlx::Type<sqlx::Postgres>
            + sqlx::Type<sqlx::Sqlite>
            + Send
            + Sync,
    ) -> Result<Option<Self>>
    where
        E: DbExecutor,
    {
        crate::crud::find_by_id::<Self, E>(executor, id).await
    }

    /// 根据多个 ID 查找记录
    async fn find_by_ids<I, E>(executor: &mut E, ids: I) -> Result<Vec<Self>>
    where
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
        E: DbExecutor,
    {
        crate::crud::find_by_ids::<Self, I, E>(executor, ids).await
    }

    /// 插入记录
    async fn insert<E>(&self, executor: &mut E) -> Result<Id>
    where
        E: DbExecutor,
    {
        crate::crud::insert::<Self, E>(self, executor).await
    }

    /// 更新记录（Patch 语义）
    ///
    /// - 非 `Option` 字段：始终参与更新，生成 `SET col = ?` 并绑定当前值。
    /// - `Option` 字段：
    ///   - `Some(v)`：生成 `SET col = ?` 并绑定 `v`；
    ///   - `None`：不生成对应的 `SET` 子句，即**不修改该列**，保留数据库中的原值。
    ///
    /// 默认实现委托给 `crate::crud::update`，具体 SQL 由 `derive(CRUD)` 宏生成。
    async fn update<E>(&self, executor: &mut E) -> Result<()>
    where
        E: DbExecutor,
    {
        crate::crud::update::<Self, E>(self, executor).await
    }

    /// 更新记录（包含 None 字段的重置，Reset 语义）
    ///
    /// - 非 Option 字段：与 `update` 相同，始终参与更新
    /// - Option 字段：
    ///   - Some(v)：更新为 v
    ///   - None：更新为数据库默认值（等价于 `SET col = DEFAULT`，具体行为由数据库决定）
    ///
    /// 默认实现委托给 `crate::crud::update_with_none`，实际 SQL 由 `derive(CRUD)` 宏生成。
    async fn update_with_none<E>(&self, executor: &mut E) -> Result<()>
    where
        E: DbExecutor,
    {
        crate::crud::update_with_none::<Self, E>(self, executor).await
    }

    /// 根据 ID 删除记录
    /// 如果指定了 SOFT_DELETE_FIELD，则进行逻辑删除；否则进行物理删除
    async fn delete_by_id<E>(
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
        E: DbExecutor,
    {
        if Self::SOFT_DELETE_FIELD.is_some() {
            crate::crud::soft_delete_by_id::<Self, E>(executor, id).await
        } else {
            crate::crud::hard_delete_by_id::<Self, E>(executor, id).await
        }
    }

    /// 根据 ID 进行逻辑删除（将逻辑删除字段设置为 1）
    async fn soft_delete_by_id<E>(
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
        E: DbExecutor,
    {
        crate::crud::soft_delete_by_id::<Self, E>(executor, id).await
    }

    /// 根据 ID 进行物理删除（真正删除记录）
    async fn hard_delete_by_id<E>(
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
        E: DbExecutor,
    {
        crate::crud::hard_delete_by_id::<Self, E>(executor, id).await
    }

    /// 分页查询
    async fn paginate<E>(
        executor: &mut E,
        builder: QueryBuilder,
        page: u64,
        size: u64,
    ) -> Result<Page<Self>>
    where
        E: DbExecutor,
    {
        crate::crud::paginate::<Self, E>(executor, builder, page, size).await
    }

    /// 安全查询所有记录（限制最多 1000 条）
    /// 如果指定了 SOFT_DELETE_FIELD，自动过滤已删除的记录
    ///
    /// # 参数
    /// * `executor` - 数据库执行器（DbPool 或 Transaction）
    /// * `builder` - 可选的查询构建器，如果为 None，则查询所有记录
    ///
    /// # 返回
    /// 返回最多 1000 条记录的向量
    async fn find_all<E>(executor: &mut E, builder: Option<QueryBuilder>) -> Result<Vec<Self>>
    where
        E: DbExecutor,
    {
        crate::crud::find_all::<Self, E>(executor, builder).await
    }

    /// 查询单条记录（使用 QueryBuilder）
    /// 如果指定了 SOFT_DELETE_FIELD，自动过滤已删除的记录
    /// 自动添加 LIMIT 1 限制
    ///
    /// # 参数
    /// * `executor` - 数据库执行器（DbPool 或 Transaction）
    /// * `builder` - 查询构建器
    ///
    /// # 返回
    /// 返回单条记录，如果未找到则返回 None
    async fn find_one<E>(executor: &mut E, builder: QueryBuilder) -> Result<Option<Self>>
    where
        E: DbExecutor,
    {
        crate::crud::find_one::<Self, E>(executor, builder).await
    }

    /// 统计记录数量（使用 QueryBuilder）
    /// 如果指定了 SOFT_DELETE_FIELD，自动过滤已删除的记录
    ///
    /// # 参数
    /// * `executor` - 数据库执行器（DbPool 或 Transaction）
    /// * `builder` - 查询构建器
    ///
    /// # 返回
    /// 返回符合条件的记录数量
    async fn count<E>(executor: &mut E, builder: QueryBuilder) -> Result<u64>
    where
        E: DbExecutor,
    {
        crate::crud::count::<Self, E>(executor, builder).await
    }
}
