use anyhow::Result;

/// 表信息
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

/// 列信息
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub sql_type: String,
    pub nullable: bool,
    pub is_pk: bool,
    pub default: Option<String>,
}

impl TableInfo {
    /// 检测逻辑删除字段（常见的命名：is_del, is_deleted, deleted_at, deleted）
    pub fn detect_soft_delete_field(&self) -> Option<&str> {
        let common_names = ["is_del", "is_deleted", "deleted_at", "deleted", "is_delete"];
        for col in &self.columns {
            if common_names.contains(&col.name.as_str()) {
                // 检查类型是否适合逻辑删除（通常是整数或布尔值）
                let sql_type_upper = col.sql_type.to_uppercase();
                if sql_type_upper.contains("INT")
                    || sql_type_upper.contains("BOOL")
                    || sql_type_upper.contains("BOOLEAN")
                    || sql_type_upper.contains("TINYINT")
                    || sql_type_upper.contains("BIT")
                {
                    return Some(&col.name);
                }
            }
        }
        None
    }
}

/// 代码生成器
pub struct CodeGenerator {
    pub serde: bool,
    pub derive_crud: bool,
}

impl CodeGenerator {
    pub fn new(serde: bool, derive_crud: bool) -> Self {
        Self { serde, derive_crud }
    }

    /// 生成模型代码
    pub fn generate_model(&self, table: &TableInfo) -> Result<String> {
        let mut code = String::new();

        // 生成结构体文档注释
        code.push_str(&format!("/// {}\n", to_pascal_case(&table.name)));
        code.push_str(&format!("/// \n"));
        code.push_str(&format!("/// 表名: `{}`\n", table.name));

        let pk = table
            .columns
            .iter()
            .find(|c| c.is_pk)
            .map(|c| c.name.as_str())
            .unwrap_or("id");
        code.push_str(&format!("/// 主键: `{}`\n", pk));

        if let Some(soft_delete) = table.detect_soft_delete_field() {
            code.push_str(&format!("/// 逻辑删除字段: `{}`\n", soft_delete));
        }
        code.push_str(&format!("/// 字段数: {}\n", table.columns.len()));
        code.push_str("\n");

        // 生成 derives
        let mut derives = vec![
            "Debug".to_string(),
            "Default".to_string(),
            "sqlx::FromRow".to_string(),
        ];
        if self.serde {
            derives.push("serde::Serialize".to_string());
            derives.push("serde::Deserialize".to_string());
        }
        derives.push("sqlxplus::ModelMeta".to_string());
        if self.derive_crud {
            derives.push("sqlxplus::CRUD".to_string());
        }

        code.push_str(&format!("#[derive({})]\n", derives.join(", ")));

        // 生成 model 属性
        let soft_delete_field = table.detect_soft_delete_field();
        if let Some(soft_delete) = soft_delete_field {
            code.push_str(&format!(
                "#[model(table = \"{}\", pk = \"{}\", soft_delete = \"{}\")]\n",
                table.name, pk, soft_delete
            ));
        } else {
            code.push_str(&format!(
                "#[model(table = \"{}\", pk = \"{}\")]\n",
                table.name, pk
            ));
        }

        code.push_str(&format!("pub struct {} {{\n", to_pascal_case(&table.name)));

        // 生成字段（带注释）
        for col in &table.columns {
            // 生成字段注释
            code.push_str("    /// ");

            // 字段描述
            let mut desc_parts = Vec::new();

            // 主键标识
            if col.is_pk {
                desc_parts.push("主键".to_string());
            }

            // 字段名和类型
            desc_parts.push(format!("{} ({})", col.name, col.sql_type));

            // 可空性
            if col.nullable {
                desc_parts.push("可空".to_string());
            } else {
                desc_parts.push("非空".to_string());
            }

            code.push_str(&desc_parts.join(" | "));
            code.push_str("\n");

            // 如果有默认值，也加上
            if let Some(ref default) = col.default {
                if default != "NULL" {
                    code.push_str(&format!("    /// 默认值: {}\n", default));
                }
            }

            // 主键 id 字段强制生成为 Option<i64>，以兼容 MySQL/PostgreSQL 的 BIGINT
            let rust_type = if col.is_pk && col.name == "id" {
                "Option<i64>".to_string()
            } else {
                sql_type_to_rust(col)
            };
            let field_name = escape_rust_keyword(&col.name);
            code.push_str(&format!("    pub {}: {},\n", field_name, rust_type));
        }

        code.push_str("}\n");
        Ok(code)
    }

    /// 生成 mod.rs（当前未使用，保留以备后用）
    #[allow(dead_code)]
    pub fn generate_mod_rs(&self, tables: &[TableInfo]) -> Result<String> {
        let mut code = String::new();
        code.push_str("// Auto-generated module file\n\n");

        for table in tables {
            let mod_name = to_snake_case(&table.name);
            let struct_name = to_pascal_case(&table.name);
            code.push_str(&format!("pub mod {};\n", mod_name));
            code.push_str(&format!("pub use {}::{};\n\n", mod_name, struct_name));
        }

        Ok(code)
    }
}

/// SQL 类型转换为 Rust 类型
///
/// 规则：
/// - 数据库字段为 NULLable 或 有默认值 时，生成 `Option<T>`
///   - 有默认值的字段往往在插入时可以不手动赋值，因此也按可选处理
/// - 否则生成裸类型 `T`
fn sql_type_to_rust(col: &ColumnInfo) -> String {
    // 规范化 SQL 类型（移除长度限制等）
    let normalized = col
        .sql_type
        .split('(')
        .next()
        .unwrap_or(&col.sql_type)
        .trim();

    let base_type = match normalized.to_uppercase().as_str() {
        // 整数类型
        // 注意：为了跨数据库兼容性，MySQL 的 TINYINT 映射到 i16（对应 PostgreSQL 的 SMALLINT）
        // MySQL 的 TINYINT 范围是 -128 到 127，i16 完全可以容纳
        "BIGINT" | "BIGSERIAL" => "i64",
        "INT" | "INTEGER" | "INT4" | "SERIAL" => "i32",
        "SMALLINT" | "INT2" | "SMALLSERIAL" | "TINYINT" => "i16",
        // 无符号整数（MySQL）
        // 为了兼容性，TINYINT UNSIGNED 映射到 u16（对应 PostgreSQL 的 SMALLINT）
        "BIGINT UNSIGNED" => "u64",
        "INT UNSIGNED" | "INTEGER UNSIGNED" => "u32",
        "SMALLINT UNSIGNED" | "TINYINT UNSIGNED" => "u16",
        // 字符串类型
        "VARCHAR" | "TEXT" | "CHAR" | "CHARACTER VARYING" | "CHARACTER" | "LONGTEXT"
        | "MEDIUMTEXT" | "TINYTEXT" | "NVARCHAR" | "NCHAR" => "String",
        // 数值类型
        "DECIMAL" | "NUMERIC" | "DOUBLE PRECISION" | "REAL" | "DOUBLE" => "f64",
        "FLOAT" | "FLOAT4" => "f32",
        "MONEY" => "f64",
        // 布尔类型
        "BOOLEAN" | "BOOL" | "BIT" => "bool",
        // 日期时间类型
        // MySQL 的 TIMESTAMP 是带时区的，应该使用 DateTime<Utc>
        // MySQL 的 DATETIME 是无时区的，应该使用 NaiveDateTime
        // PostgreSQL 的 TIMESTAMP WITHOUT TIME ZONE 是无时区的，使用 NaiveDateTime
        // PostgreSQL 的 TIMESTAMP WITH TIME ZONE 是带时区的，使用 DateTime<Utc>
        "DATE" => "chrono::NaiveDate",
        "TIME" | "TIME WITHOUT TIME ZONE" => "chrono::NaiveTime",
        "DATETIME" => "chrono::NaiveDateTime",
        "TIMESTAMP WITHOUT TIME ZONE" => "chrono::NaiveDateTime",
        "TIMESTAMP WITH TIME ZONE" | "TIMESTAMPTZ" => "chrono::DateTime<chrono::Utc>",
        "TIMESTAMP" => {
            // MySQL 的 TIMESTAMP 使用 DateTime<Utc>（带时区）
            // PostgreSQL 的 TIMESTAMP 如果没有指定时区，默认是 TIMESTAMP WITHOUT TIME ZONE，使用 NaiveDateTime
            // 但为了兼容性，MySQL 的 TIMESTAMP 统一使用 DateTime<Utc>
            // 检查原始类型字符串，如果包含 "WITHOUT TIME ZONE" 则使用 NaiveDateTime
            let sql_type_upper = col.sql_type.to_uppercase();
            if sql_type_upper.contains("WITHOUT TIME ZONE") {
                "chrono::NaiveDateTime"
            } else {
                // MySQL 的 TIMESTAMP 使用 DateTime<Utc>
                "chrono::DateTime<chrono::Utc>"
            }
        }
        // 二进制类型
        "BLOB" | "BYTEA" | "BINARY" | "VARBINARY" | "LONGBLOB" | "MEDIUMBLOB" | "TINYBLOB" => {
            "Vec<u8>"
        }
        // JSON 类型
        "JSON" | "JSONB" => "serde_json::Value",
        // UUID
        "UUID" => "uuid::Uuid",
        // 默认返回 String
        _ => "String",
    };

    // 有默认值 或 可空 字段，统一生成为 Option<T>
    let needs_option = col.nullable || col.default.is_some();

    if needs_option {
        format!("Option<{}>", base_type)
    } else {
        base_type.to_string()
    }
}

/// 转换为 PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// 转换为 snake_case（当前未使用，保留以备后用）
#[allow(dead_code)]
fn to_snake_case(s: &str) -> String {
    s.to_lowercase()
}

/// 如果字段名是 Rust 关键字，使用原始标识符前缀避免编译错误
fn escape_rust_keyword(name: &str) -> String {
    if is_rust_keyword(name) {
        format!("r#{}", name)
    } else {
        name.to_string()
    }
}

/// 判断是否为 Rust 关键字
fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        "as"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            // 未来/保留关键字
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}
