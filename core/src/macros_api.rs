//! 宏接口模块，供 proc-macro crate 使用
//!
//! 这个模块提供了 derive 宏生成代码时需要的辅助函数和类型
//! 
/// 字段元数据，由 derive(ModelMeta) 生成
pub struct FieldMeta {
    pub name: &'static str,
    pub column: &'static str,
    pub skip: bool,
    /// 是否创建索引：None 表示不创建索引，Some(name) 表示创建索引并使用指定名称，如果 name 为空则使用默认名称
    pub index: Option<&'static str>,
    /// 是否创建联合索引：None 表示不创建联合索引，Some((name, order)) 表示加入名为 name 的联合索引，order 指定在联合索引中的顺序（数字越小越靠前）
    pub combine_index: Option<(&'static str, i32)>,
    /// 是否创建唯一索引
    pub unique: bool,
    /// 是否非空（即使类型是 Option<T>，如果设置了 not_null，也会生成 NOT NULL）
    pub not_null: bool,
    /// 默认值（SQL 表达式字符串，如 "CURRENT_TIMESTAMP(3)"、"0" 等）
    pub default: Option<&'static str>,
    /// 字段长度（用于 VARCHAR 等类型）
    pub length: Option<u32>,
    /// 是否自增（MySQL AUTO_INCREMENT）
    pub auto_increment: bool,
    /// 是否主键
    pub primary_key: bool,
    /// 是否逻辑删除
    pub soft_delete: bool,
    /// 字段注释
    pub comment: Option<&'static str>,
}

/// 模型元数据，由 derive(ModelMeta) 生成
pub struct ModelMeta {
    /// 表名
    pub table: &'static str,
    /// 主键字段名
    pub pk: &'static str,
    /// 逻辑删除字段名
    pub soft_delete: Option<&'static str>,
    /// 字段列表
    pub fields: &'static [FieldMeta],
    /// 表注释
    pub table_comment: Option<&'static str>,
}
