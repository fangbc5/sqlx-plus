use anyhow::Result;
use std::collections::HashMap;

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

        // 生成 derives
        let mut derives = vec!["sqlx::FromRow".to_string()];
        if self.serde {
            derives.push("serde::Serialize".to_string());
            derives.push("serde::Deserialize".to_string());
        }
        derives.push("sqlx_plus_derive::ModelMeta".to_string());
        if self.derive_crud {
            derives.push("sqlx_plus_derive::CRUD".to_string());
        }

        code.push_str(&format!("#[derive({})]\n", derives.join(", ")));
        code.push_str(&format!("#[model(table = \"{}\", pk = \"{}\")]\n", 
            table.name, 
            table.columns.iter().find(|c| c.is_pk).map(|c| &c.name).unwrap_or("id")
        ));
        code.push_str(&format!("pub struct {} {{\n", to_pascal_case(&table.name)));

        // 生成字段
        for col in &table.columns {
            let rust_type = sql_type_to_rust(&col.sql_type, col.nullable);
            code.push_str(&format!("    pub {}: {},\n", col.name, rust_type));
        }

        code.push_str("}\n");
        Ok(code)
    }

    /// 生成 mod.rs
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
fn sql_type_to_rust(sql_type: &str, nullable: bool) -> String {
    let base_type = match sql_type.to_uppercase().as_str() {
        "INT" | "INTEGER" | "BIGINT" => "i64",
        "INT UNSIGNED" | "BIGINT UNSIGNED" => "u64",
        "SMALLINT" | "TINYINT" => "i32",
        "VARCHAR" | "TEXT" | "CHAR" | "LONGTEXT" | "MEDIUMTEXT" => "String",
        "DECIMAL" | "NUMERIC" | "FLOAT" | "DOUBLE" => "f64",
        "BOOLEAN" | "BOOL" | "TINYINT(1)" => "bool",
        "DATE" | "DATETIME" | "TIMESTAMP" => "chrono::NaiveDateTime",
        "TIME" => "chrono::NaiveTime",
        _ => "String",
    };

    if nullable {
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

/// 转换为 snake_case
fn to_snake_case(s: &str) -> String {
    s.to_lowercase()
}

