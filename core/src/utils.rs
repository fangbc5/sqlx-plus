//! 工具函数模块

use crate::db_pool::DbDriver;

/// SQL 类型到 Rust 类型的映射
pub fn sql_type_to_rust(sql_type: &str, nullable: bool) -> String {
    let base_type = match sql_type.to_uppercase().as_str() {
        "INT" | "INTEGER" | "BIGINT" => "i64",
        "INT UNSIGNED" | "BIGINT UNSIGNED" => "u64",
        "SMALLINT" | "TINYINT" => "i32",
        "VARCHAR" | "TEXT" | "CHAR" | "LONGTEXT" => "String",
        "DECIMAL" | "NUMERIC" | "FLOAT" | "DOUBLE" => "f64",
        "BOOLEAN" | "BOOL" | "TINYINT(1)" => "bool",
        "DATE" | "DATETIME" | "TIMESTAMP" => "chrono::NaiveDateTime",
        "TIME" => "chrono::NaiveTime",
        _ => "String", // 默认类型
    };

    if nullable {
        format!("Option<{}>", base_type)
    } else {
        base_type.to_string()
    }
}

/// 验证表名是否安全
pub fn is_safe_table_name(name: &str) -> bool {
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// 验证字段名是否安全
pub fn is_safe_field_name(name: &str) -> bool {
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// 转义 SQL 标识符
pub fn escape_identifier(driver: DbDriver, name: &str) -> String {
    match driver {
        DbDriver::MySql => format!("`{}`", name),
        DbDriver::Postgres => format!("\"{}\"", name),
        DbDriver::Sqlite => format!("\"{}\"", name),
    }
}


