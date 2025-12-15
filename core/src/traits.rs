use crate::crud::{Id, Page};
use crate::db_pool::DbPool;
use crate::db_pool::Result;
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
    async fn find_by_id(
        pool: &DbPool,
        id: impl for<'q> sqlx::Encode<'q, sqlx::MySql>
            + for<'q> sqlx::Encode<'q, sqlx::Postgres>
            + for<'q> sqlx::Encode<'q, sqlx::Sqlite>
            + sqlx::Type<sqlx::MySql>
            + sqlx::Type<sqlx::Postgres>
            + sqlx::Type<sqlx::Sqlite>
            + Send
            + Sync,
    ) -> Result<Option<Self>> {
        crate::crud::find_by_id::<Self>(pool, id).await
    }

    /// 插入记录
    async fn insert(&self, pool: &DbPool) -> Result<Id> {
        crate::crud::insert(self, pool).await
    }

    /// 更新记录
    async fn update(&self, pool: &DbPool) -> Result<()> {
        crate::crud::update(self, pool).await
    }

    /// 根据 ID 删除记录
    /// 如果指定了 SOFT_DELETE_FIELD，则进行逻辑删除；否则进行物理删除
    async fn delete_by_id(
        pool: &DbPool,
        id: impl for<'q> sqlx::Encode<'q, sqlx::MySql>
            + for<'q> sqlx::Encode<'q, sqlx::Postgres>
            + for<'q> sqlx::Encode<'q, sqlx::Sqlite>
            + sqlx::Type<sqlx::MySql>
            + sqlx::Type<sqlx::Postgres>
            + sqlx::Type<sqlx::Sqlite>
            + Send
            + Sync,
    ) -> Result<()> {
        if Self::SOFT_DELETE_FIELD.is_some() {
            crate::crud::soft_delete_by_id::<Self>(pool, id).await
        } else {
            crate::crud::hard_delete_by_id::<Self>(pool, id).await
        }
    }

    /// 根据 ID 进行逻辑删除（将逻辑删除字段设置为 1）
    async fn soft_delete_by_id(
        pool: &DbPool,
        id: impl for<'q> sqlx::Encode<'q, sqlx::MySql>
            + for<'q> sqlx::Encode<'q, sqlx::Postgres>
            + for<'q> sqlx::Encode<'q, sqlx::Sqlite>
            + sqlx::Type<sqlx::MySql>
            + sqlx::Type<sqlx::Postgres>
            + sqlx::Type<sqlx::Sqlite>
            + Send
            + Sync,
    ) -> Result<()> {
        crate::crud::soft_delete_by_id::<Self>(pool, id).await
    }

    /// 根据 ID 进行物理删除（真正删除记录）
    async fn hard_delete_by_id(
        pool: &DbPool,
        id: impl for<'q> sqlx::Encode<'q, sqlx::MySql>
            + for<'q> sqlx::Encode<'q, sqlx::Postgres>
            + for<'q> sqlx::Encode<'q, sqlx::Sqlite>
            + sqlx::Type<sqlx::MySql>
            + sqlx::Type<sqlx::Postgres>
            + sqlx::Type<sqlx::Sqlite>
            + Send
            + Sync,
    ) -> Result<()> {
        crate::crud::hard_delete_by_id::<Self>(pool, id).await
    }

    /// 分页查询
    async fn paginate(
        pool: &DbPool,
        builder: QueryBuilder,
        page: u64,
        size: u64,
    ) -> Result<Page<Self>> {
        crate::crud::paginate::<Self>(pool, builder, page, size).await
    }

    /// 安全查询所有记录（限制最多 1000 条）
    /// 如果指定了 SOFT_DELETE_FIELD，自动过滤已删除的记录
    ///
    /// # 参数
    /// * `pool` - 数据库连接池
    /// * `builder` - 可选的查询构建器，如果为 None，则查询所有记录
    ///
    /// # 返回
    /// 返回最多 1000 条记录的向量
    async fn find_all(pool: &DbPool, builder: Option<QueryBuilder>) -> Result<Vec<Self>> {
        crate::crud::find_all::<Self>(pool, builder).await
    }
}
