/// User
/// 
/// 表名: `user`
/// 主键: `id`
/// 逻辑删除字段: `is_del`
/// 字段数: 36

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "user", pk = "id", soft_delete = "is_del", table_comment = "用户表")]
pub struct User {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    /// system_type (smallint) | 非空
    /// 默认值: 1
    #[column(not_null, default = "1", comment = "系统类型")]
    pub system_type: Option<i16>,
    /// user_type (smallint) | 可空
    /// 默认值: 3
    #[column(default = "3", comment = "用户类型")]
    pub user_type: Option<i16>,
    /// username (character varying(255)) | 可空
    #[column(length = 255, index, comment = "用户名")]
    pub username: Option<String>,
    /// nick_name (character varying(255)) | 可空
    #[column(length = 255, comment = "昵称")]
    pub nick_name: Option<String>,
    /// real_name (character varying(255)) | 可空
    #[column(length = 255, comment = "真实姓名")]
    pub real_name: Option<String>,
    /// avatar (character varying(255)) | 非空
    /// 默认值: ''::character varying
    #[column(not_null, default = "", length = 255, comment = "头像URL")]
    pub avatar: Option<String>,
    /// avatar_update_time (timestamp with time zone) | 可空
    #[column(comment = "头像更新时间")]
    pub avatar_update_time: Option<chrono::DateTime<chrono::Utc>>,
    /// email (character varying(255)) | 可空
    #[column(length = 255, unique, index, comment = "邮箱地址")]
    pub email: Option<String>,
    /// region (character varying(5)) | 可空
    #[column(length = 5, comment = "地区代码")]
    pub region: Option<String>,
    /// mobile (character varying(11)) | 可空
    #[column(length = 11, comment = "手机号")]
    pub mobile: Option<String>,
    /// id_card (character varying(18)) | 可空
    #[column(length = 18, comment = "身份证号")]
    pub id_card: Option<String>,
    /// wx_open_id (character varying(255)) | 可空
    #[column(length = 255, comment = "微信OpenID")]
    pub wx_open_id: Option<String>,
    /// dd_open_id (character varying(255)) | 可空
    #[column(length = 255, comment = "钉钉OpenID")]
    pub dd_open_id: Option<String>,
    /// sex (smallint) | 可空
    /// 默认值: 0
    #[column(default = "0", comment = "性别：0-未知，1-男，2-女")]
    pub sex: Option<i16>,
    /// state (smallint) | 可空
    /// 默认值: 1
    #[column(default = "1", comment = "状态：1-正常")]
    pub state: Option<i16>,
    /// user_state_id (bigint) | 可空
    #[column(comment = "用户状态ID")]
    pub user_state_id: Option<i64>,
    /// resume (character varying(200)) | 可空
    #[column(length = 200, comment = "简历")]
    pub resume: Option<String>,
    /// work_describe (character varying(255)) | 可空
    #[column(length = 255, comment = "工作描述")]
    pub work_describe: Option<String>,
    /// item_id (bigint) | 可空
    #[column(comment = "项目ID")]
    pub item_id: Option<i64>,
    /// context (smallint) | 可空
    /// 默认值: 0
    #[column(default = "0", comment = "上下文")]
    pub context: Option<i16>,
    /// num (bigint) | 可空
    /// 默认值: 10
    #[column(default = "10", comment = "数量")]
    pub num: Option<i64>,
    /// password (character varying(64)) | 非空
    #[column(not_null, length = 64, comment = "密码")]
    pub password: Option<String>,
    /// salt (character varying(20)) | 可空
    #[column(length = 20, comment = "盐值")]
    pub salt: Option<String>,
    /// password_error_num (integer) | 可空
    /// 默认值: 0
    #[column(default = "0", comment = "密码错误次数")]
    pub password_error_num: Option<i32>,
    /// password_error_last_time (timestamp with time zone) | 可空
    #[column(comment = "密码错误最后时间")]
    pub password_error_last_time: Option<chrono::DateTime<chrono::Utc>>,
    /// password_expire_time (timestamp with time zone) | 可空
    #[column(comment = "密码过期时间")]
    pub password_expire_time: Option<chrono::DateTime<chrono::Utc>>,
    /// last_opt_time (timestamp with time zone) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(default = "CURRENT_TIMESTAMP(3)", comment = "最后操作时间")]
    pub last_opt_time: Option<chrono::DateTime<chrono::Utc>>,
    /// last_login_time (timestamp with time zone) | 可空
    #[column(comment = "最后登录时间")]
    pub last_login_time: Option<chrono::DateTime<chrono::Utc>>,
    /// ip_info (jsonb) | 可空
    #[column(comment = "IP信息")]
    pub ip_info: Option<serde_json::Value>,
    /// create_time (timestamp with time zone) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)", comment = "创建时间")]
    pub create_time: Option<chrono::DateTime<chrono::Utc>>,
    /// update_time (timestamp with time zone) | 非空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(not_null, default = "CURRENT_TIMESTAMP(3)", comment = "更新时间")]
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
    /// create_by (bigint) | 可空
    /// 默认值: 1
    #[column(default = "1", comment = "创建人ID")]
    pub create_by: Option<i64>,
    /// update_by (bigint) | 可空
    #[column(comment = "更新人ID")]
    pub update_by: Option<i64>,
    /// is_del (smallint) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", index, soft_delete, comment = "是否删除：0-未删除，1-已删除")]
    pub is_del: Option<i16>,
    /// readonly (smallint) | 可空
    /// 默认值: 0
    #[column(default = "0", comment = "是否只读：0-否，1-是")]
    pub readonly: Option<i16>,
}
