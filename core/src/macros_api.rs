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
