//! 宏接口模块，供 proc-macro crate 使用
//!
//! 这个模块提供了 derive 宏生成代码时需要的辅助函数和类型

use crate::db_pool::{DbPool, Result};

/// 字段元数据，由 derive(ModelMeta) 生成
pub struct FieldMeta {
    pub name: &'static str,
    pub column: &'static str,
    pub skip: bool,
}

/// 模型元数据，由 derive(ModelMeta) 生成
pub struct ModelMeta {
    pub table: &'static str,
    pub pk: &'static str,
    pub fields: &'static [FieldMeta],
}

/// 执行插入操作的辅助函数（供宏使用）
pub async fn execute_insert(
    _pool: &DbPool,
    _sql: &str,
    _binds: &[&dyn sqlx::Encode<'_, sqlx::MySql>],
) -> Result<crate::crud::Id> {
    // 这个函数会被宏生成的代码调用
    // 实际实现需要根据不同的数据库驱动进行适配
    todo!("This function should be implemented by the macro-generated code")
}
