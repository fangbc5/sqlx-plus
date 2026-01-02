/// User2
/// 
/// 表名: `user2`
/// 主键: `id`
/// 用于测试各种数据类型和默认值

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "user2", pk = "id", table_comment = "用户表2 - 测试各种数据类型")]
pub struct User2 {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    
    // ========== 整数类型 ==========
    /// tinyint 类型 | 非空 | 默认值: 1
    #[column(not_null, default = "1", comment = "TINYINT类型")]
    pub tinyint_field: Option<i16>,
    
    /// smallint 类型 | 可空
    #[column(comment = "SMALLINT类型")]
    pub smallint_field: Option<i16>,
    
    /// int 类型 | 非空 | 默认值: 100
    #[column(not_null, default = "100", comment = "INT类型")]
    pub int_field: Option<i32>,
    
    /// bigint 类型 | 可空
    #[column(comment = "BIGINT类型")]
    pub bigint_field: Option<i64>,
    
    // ========== 浮点数类型 ==========
    /// float 类型 | 非空 | 默认值: 3.14
    #[column(not_null, default = "3.14", comment = "FLOAT类型")]
    pub float_field: Option<f32>,
    
    /// double 类型 | 可空 | 默认值: 2.71828
    #[column(default = "2.71828", comment = "DOUBLE类型")]
    pub double_field: Option<f64>,
    
    // ========== 布尔类型 ==========
    /// bool 类型 | 非空 | 默认值: true (测试各种格式)
    #[column(not_null, default = "b\'1\'", comment = "BOOL类型 - 默认true")]
    pub bool_field_true: Option<bool>,
    
    /// bool 类型 | 非空 | 默认值: false
    #[column(not_null, default = "0", comment = "BOOL类型 - 默认false")]
    pub bool_field_false: Option<bool>,
    
    /// bool 类型 | 可空 | 默认值: true
    #[column(default = "true", comment = "BOOL类型 - 可空")]
    pub bool_field_nullable: Option<bool>,
    
    // ========== 字符串类型 ==========
    /// varchar 类型 | 非空 | 长度: 50
    #[column(not_null, length = 50, comment = "VARCHAR类型 - 非空")]
    pub varchar_not_null: Option<String>,
    
    /// varchar 类型 | 可空 | 长度: 100 | 默认值: 空字符串
    #[column(length = 100, default = "", comment = "VARCHAR类型 - 可空")]
    pub varchar_nullable: Option<String>,
    
    /// text 类型 | 可空
    #[column(comment = "TEXT类型")]
    pub text_field: Option<String>,
    
    // ========== 日期时间类型 ==========
    /// date 类型 | 可空
    #[column(comment = "DATE类型")]
    pub date_field: Option<chrono::NaiveDate>,
    
    /// datetime 类型 | 非空 | 默认值: CURRENT_TIMESTAMP
    #[column(not_null, default = "CURRENT_TIMESTAMP", comment = "DATETIME类型")]
    pub datetime_field: Option<chrono::NaiveDateTime>,
    
    /// timestamp 类型 | 非空 | 默认值: CURRENT_TIMESTAMP
    #[column(not_null, default = "CURRENT_TIMESTAMP", comment = "TIMESTAMP类型")]
    pub timestamp_field: Option<chrono::DateTime<chrono::Utc>>,
    
    /// time 类型 | 可空
    #[column(comment = "TIME类型")]
    pub time_field: Option<chrono::NaiveTime>,
    
    // ========== JSON 类型 ==========
    /// json 类型 | 可空
    #[column(comment = "JSON类型")]
    pub json_field: Option<serde_json::Value>,
    
    // ========== 二进制类型 ==========
    /// blob 类型 | 可空
    #[column(comment = "BLOB类型")]
    pub blob_field: Option<Vec<u8>>,
    
    // ========== 索引和约束测试 ==========
    /// 唯一索引字段
    #[column(unique, length = 50, comment = "唯一索引字段")]
    pub unique_field: Option<String>,
    
    /// 普通索引字段
    #[column(index, length = 50, comment = "普通索引字段")]
    pub index_field: Option<String>,
    
    /// 联合索引字段1
    #[column(combine_index = "idx_composite:1", length = 50, comment = "联合索引字段1")]
    pub composite_field1: Option<String>,
    
    /// 联合索引字段2
    #[column(combine_index = "idx_composite:2", length = 50, comment = "联合索引字段2")]
    pub composite_field2: Option<String>,
}

