/// User
/// 
/// 表名: `user`
/// 主键: `id`
/// 逻辑删除字段: `is_del`
/// 字段数: 37

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "user", pk = "id", soft_delete = "is_del")]
pub struct User {
    /// 主键 | id (bigint) | 非空
    pub id: Option<i64>,
    /// system_type (tinyint) | 非空
    /// 默认值: 1
    pub system_type: Option<i16>,
    /// user_type (tinyint) | 可空
    /// 默认值: 3
    pub user_type: Option<i16>,
    /// username (varchar(255)) | 可空
    pub username: Option<String>,
    /// nick_name (varchar(255)) | 可空
    pub nick_name: Option<String>,
    /// real_name (varchar(255)) | 可空
    pub real_name: Option<String>,
    /// avatar (varchar(255)) | 非空
    /// 默认值: 
    pub avatar: Option<String>,
    /// avatar_update_time (timestamp(3)) | 可空
    pub avatar_update_time: Option<chrono::DateTime<chrono::Utc>>,
    /// email (varchar(255)) | 可空
    pub email: Option<String>,
    /// region (varchar(5)) | 可空
    pub region: Option<String>,
    /// mobile (varchar(11)) | 可空
    pub mobile: Option<String>,
    /// id_card (varchar(18)) | 可空
    pub id_card: Option<String>,
    /// wx_open_id (varchar(255)) | 可空
    pub wx_open_id: Option<String>,
    /// dd_open_id (varchar(255)) | 可空
    pub dd_open_id: Option<String>,
    /// sex (tinyint) | 可空
    /// 默认值: 0
    pub sex: Option<i16>,
    /// state (tinyint) | 可空
    /// 默认值: 1
    pub state: Option<i16>,
    /// user_state_id (bigint) | 可空
    pub user_state_id: Option<i64>,
    /// resume (varchar(200)) | 可空
    pub resume: Option<String>,
    /// work_describe (varchar(255)) | 可空
    pub work_describe: Option<String>,
    /// item_id (bigint) | 可空
    pub item_id: Option<i64>,
    /// context (tinyint) | 可空
    /// 默认值: 0
    pub context: Option<i16>,
    /// num (bigint) | 可空
    /// 默认值: 10
    pub num: Option<i64>,
    /// password (varchar(64)) | 非空
    /// 默认值: 
    pub password: Option<String>,
    /// salt (varchar(20)) | 可空
    pub salt: Option<String>,
    /// password_error_num (int) | 可空
    /// 默认值: 0
    pub password_error_num: Option<i32>,
    /// password_error_last_time (timestamp) | 可空
    pub password_error_last_time: Option<chrono::DateTime<chrono::Utc>>,
    /// password_expire_time (timestamp) | 可空
    pub password_expire_time: Option<chrono::DateTime<chrono::Utc>>,
    /// last_opt_time (timestamp(3)) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    pub last_opt_time: Option<chrono::DateTime<chrono::Utc>>,
    /// last_login_time (timestamp) | 可空
    pub last_login_time: Option<chrono::DateTime<chrono::Utc>>,
    /// ip_info (json) | 可空
    pub ip_info: Option<serde_json::Value>,
    /// create_time (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    pub create_time: Option<chrono::DateTime<chrono::Utc>>,
    /// update_time (timestamp(3)) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
    /// create_by (bigint) | 可空
    /// 默认值: 1
    pub create_by: Option<i64>,
    /// update_by (bigint) | 可空
    pub update_by: Option<i64>,
    /// is_del (tinyint) | 非空
    /// 默认值: 0
    pub is_del: Option<i16>,
    /// readonly (tinyint) | 可空
    /// 默认值: 0
    pub readonly: Option<i16>,
}
