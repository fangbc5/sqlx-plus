/// User2
/// 
/// 表名: `user2`
/// 主键: `id`
/// 用于测试组合索引功能

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "user2", pk = "id")]
pub struct User2 {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment)]
    pub id: Option<i64>,
    
    /// system_type (tinyint) | 非空
    /// 默认值: 1
    /// 既有单独索引，又加入联合索引（顺序为1）
    #[column(not_null, default = "1", index, combine_index)]
    pub system_type: Option<i16>,
    
    /// is_del (tinyint) | 非空
    /// 默认值: 0
    /// 加入联合索引（顺序为2）
    #[column(not_null, default = "0", combine_index)]
    pub is_del: Option<i16>,
    
    /// username (varchar(255)) | 可空
    /// 使用默认索引名称
    #[column(unique, length = 255, comment = "用户名")]
    pub username: Option<String>,
    
    /// email (varchar(255)) | 可空
    /// 唯一索引，同时加入联合索引（顺序为1）
    #[column(unique, index = "idx_email", combine_index = "idx_email_status:1", length = 255, comment = "邮箱地址")]
    pub email: Option<String>,
    
    /// status (tinyint) | 可空
    /// 默认值: 1
    /// 加入联合索引（顺序为2）
    #[column(default = "1", combine_index = "idx_email_status:2")]
    pub status: Option<i16>,
    
    /// create_time (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)")]
    pub create_time: Option<chrono::DateTime<chrono::Utc>>,
    
    /// update_time (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)")]
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
}

