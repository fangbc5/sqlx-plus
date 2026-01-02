use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use syn::{parse_file, Fields, Type, TypePath};
use syn::parse::Parser;

/// 从 Rust 模型文件生成建表 SQL
pub struct SqlGenerator;

impl SqlGenerator {
    /// 从模型文件生成建表 SQL
    pub fn generate_create_table(
        model_file: &Path,
        database: &str,
    ) -> Result<String> {
        // 读取文件内容
        let content = fs::read_to_string(model_file)
            .with_context(|| format!("Failed to read file: {:?}", model_file))?;

        // 解析 Rust 代码
        let ast = parse_file(&content)
            .context("Failed to parse Rust file")?;

        // 查找带有 #[model(...)] 属性的结构体
        let mut create_statements = Vec::new();
        for item in ast.items {
            if let syn::Item::Struct(item_struct) = item {
                // 检查是否有 #[model(...)] 属性
                let model_attr = item_struct
                    .attrs
                    .iter()
                    .find(|attr| attr.path().is_ident("model"));

                if let Some(attr) = model_attr {
                    let sql = Self::generate_table_sql(&item_struct, attr, database)?;
                    create_statements.push(sql);
                }
            }
        }

        if create_statements.is_empty() {
            anyhow::bail!("No model struct found in the file");
        }

        Ok(create_statements.join("\n\n"))
    }

    /// 生成单个表的 CREATE TABLE 语句
    fn generate_table_sql(
        struct_item: &syn::ItemStruct,
        model_attr: &syn::Attribute,
        database: &str,
    ) -> Result<String> {
        // 解析 #[model(...)] 属性
        let (table_name, pk_field, _soft_delete_field, table_comment) = Self::parse_model_attr(model_attr)?;

        // 获取字段
        let fields = match &struct_item.fields {
            Fields::Named(fields) => &fields.named,
            _ => anyhow::bail!("Only named fields are supported"),
        };

        // 生成列定义
        let mut column_defs = Vec::new();
        let mut indexes: Vec<(String, String)> = Vec::new(); // 单字段索引：(索引名, 字段名)
        let mut unique_indexes: Vec<(String, String)> = Vec::new(); // 单字段唯一索引：(索引名, 字段名)
        // 联合索引：索引名 -> (字段名, 顺序) 列表
        let mut composite_indexes: std::collections::HashMap<String, Vec<(String, i32)>> = std::collections::HashMap::new();
        // PostgreSQL 的字段注释：(字段名, 注释)
        let mut postgres_comments: Vec<(String, String)> = Vec::new();
        
        for (field_index, field) in fields.iter().enumerate() {
            let field_name = field.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();

            // 检查是否有 #[skip] 属性
            let skip = field.attrs.iter().any(|attr| attr.path().is_ident("skip"));
            if skip {
                continue;
            }

            // 解析字段的 column 属性
            let column_meta = Self::parse_column_attr(&field.attrs)?;
            
            // 从字段类型推断 SQL 类型
            let mut sql_type = Self::rust_type_to_sql(&field.ty, database)?;
            
            // 如果指定了 length，更新 SQL 类型（主要用于 VARCHAR）
            // 注意：这个检查要在 TEXT 类型判断之前，因为如果指定了 length，就不应该使用 TEXT
            if let Some(length) = column_meta.length {
                if sql_type.starts_with("VARCHAR") {
                    sql_type = format!("VARCHAR({})", length);
                } else if sql_type.starts_with("CHAR") {
                    sql_type = format!("CHAR({})", length);
                }
            } else {
                // 对于 String 类型，如果没有指定 length 且字段名包含 "text"，使用 TEXT 类型
                if sql_type.starts_with("VARCHAR") {
                    let field_name_lower = field_name_str.to_lowercase();
                    if field_name_lower.contains("text") || field_name_lower.ends_with("_text") {
                        // 根据数据库类型选择 TEXT 类型
                        sql_type = match database {
                            "mysql" => "TEXT".to_string(),
                            "postgres" => "TEXT".to_string(),
                            "sqlite" => "TEXT".to_string(),
                            _ => "TEXT".to_string(),
                        };
                    }
                }
            }
            
            // 判断是否可空（Option<T> 类型是可空的，但如果设置了 not_null，则不可空）
            let nullable = Self::is_option_type(&field.ty) && !column_meta.not_null;
            
            // 判断是否是主键
            let is_pk = field_name_str == pk_field;

            // 生成列定义
            let mut col_def = format!("    {}", Self::escape_identifier(database, &field_name_str));
            col_def.push_str(&format!(" {}", sql_type));

            // 非空约束
            if !nullable && !is_pk {
                col_def.push_str(" NOT NULL");
            } else if column_meta.not_null && !is_pk {
                col_def.push_str(" NOT NULL");
            }

            // 默认值处理
            if let Some(ref default) = column_meta.default {
                let default_sql = Self::format_default_value(
                    default,
                    &field.ty,
                    &sql_type,
                    database,
                )?;
                col_def.push_str(&format!(" DEFAULT {}", default_sql));
            }

            // 主键字段如果是自增的，添加 AUTO_INCREMENT (MySQL) 或 SERIAL (PostgreSQL)
            if is_pk {
                match database {
                    "mysql" => {
                        // MySQL 的 BIGINT 主键通常是自增的，或者如果设置了 auto_increment
                        if sql_type.contains("BIGINT") || column_meta.auto_increment {
                            col_def.push_str(" AUTO_INCREMENT");
                        }
                    }
                    "postgres" => {
                        // PostgreSQL 使用 SERIAL 或 BIGSERIAL
                        if sql_type.contains("BIGINT") {
                            col_def = format!("    {} BIGSERIAL", Self::escape_identifier(database, &field_name_str));
                        } else if sql_type.contains("INT") {
                            col_def = format!("    {} SERIAL", Self::escape_identifier(database, &field_name_str));
                        }
                    }
                    _ => {}
                }
            } else if column_meta.auto_increment {
                // 非主键字段也可以设置自增（MySQL）
                if database == "mysql" {
                    col_def.push_str(" AUTO_INCREMENT");
                }
            }

            // 添加字段注释
            // MySQL 支持在 CREATE TABLE 中直接添加 COMMENT
            // PostgreSQL 需要在 CREATE TABLE 之后单独执行 COMMENT ON COLUMN
            // SQLite 不支持元数据 COMMENT，但可以使用 SQL 注释（--）
            if let Some(ref comment) = column_meta.comment {
                match database {
                    "mysql" => {
                        // MySQL: COMMENT 'comment text'
                        // 需要转义单引号
                        let escaped_comment = comment.replace('\'', "''");
                        col_def.push_str(&format!(" COMMENT '{}'", escaped_comment));
                    }
                    "postgres" => {
                        // PostgreSQL 的 COMMENT 会在 CREATE TABLE 之后单独生成
                        // 收集注释信息，稍后处理
                        postgres_comments.push((field_name_str.clone(), comment.clone()));
                    }
                    "sqlite" => {
                        // SQLite 不支持元数据 COMMENT，但可以使用 SQL 注释
                        // 在列定义后添加注释
                        col_def.push_str(&format!(" -- {}", comment));
                    }
                    _ => {
                        // 其他数据库不支持 COMMENT，忽略
                    }
                }
            }

            column_defs.push(col_def);

            // 收集索引信息
            // 1. 处理联合索引（combine_index）
            // 注意：unique 属性只影响单独索引，不影响联合索引
            // 如果需要唯一联合索引，需要所有参与字段都标记为 unique（或者后续可以添加专门的属性）
            if let Some((ref combine_index_name, order)) = column_meta.combine_index {
                // 如果顺序是 i32::MAX，表示未指定顺序，使用字段在结构体中的位置
                let final_order = if order == i32::MAX {
                    field_index as i32
                } else {
                    order
                };
                
                // 联合索引默认不是唯一的（unique 属性只影响单独索引）
                composite_indexes
                    .entry(combine_index_name.clone())
                    .or_insert_with(Vec::new)
                    .push((field_name_str.clone(), final_order));
            }
            
            // 2. 处理单独索引
            // 逻辑：
            // - 如果设置了 unique，创建唯一索引（不管是否有 index）
            // - 如果只设置了 index（没有 unique），创建普通索引
            if column_meta.unique {
                // 有 unique，创建唯一索引
                let final_index_name = if let Some(ref index_name) = column_meta.index {
                    if index_name.is_empty() {
                        // 使用默认唯一索引名称
                        format!("uk_{}_{}", table_name, field_name_str)
                    } else {
                        index_name.clone()
                    }
                } else {
                    // 没有指定 index，使用默认唯一索引名称
                    format!("uk_{}_{}", table_name, field_name_str)
                };
                unique_indexes.push((final_index_name, field_name_str.clone()));
            } else if let Some(ref index_name) = column_meta.index {
                // 只有 index，没有 unique，创建普通索引
                let final_index_name = if index_name.is_empty() {
                    // 使用默认索引名称
                    format!("idx_{}_{}", table_name, field_name_str)
                } else {
                    index_name.clone()
                };
                indexes.push((final_index_name, field_name_str.clone()));
            }
        }

        // 生成 PRIMARY KEY 约束
        let mut constraints = Vec::new();
        constraints.push(format!(
            "    PRIMARY KEY ({})",
            Self::escape_identifier(database, &pk_field)
        ));

        // 生成单字段唯一索引约束
        // MySQL 使用 UNIQUE KEY，PostgreSQL 使用 CONSTRAINT ... UNIQUE，SQLite 使用 UNIQUE
        for (index_name, field_name) in &unique_indexes {
            match database {
                "mysql" => {
                    constraints.push(format!(
                        "    UNIQUE KEY {} ({})",
                        Self::escape_identifier(database, index_name),
                        Self::escape_identifier(database, field_name)
                    ));
                }
                "postgres" => {
                    constraints.push(format!(
                        "    CONSTRAINT {} UNIQUE ({})",
                        Self::escape_identifier(database, index_name),
                        Self::escape_identifier(database, field_name)
                    ));
                }
                "sqlite" => {
                    constraints.push(format!(
                        "    UNIQUE ({})",
                        Self::escape_identifier(database, field_name)
                    ));
                }
                _ => {
                    // 默认使用 MySQL 语法
                    constraints.push(format!(
                        "    UNIQUE KEY {} ({})",
                        Self::escape_identifier(database, index_name),
                        Self::escape_identifier(database, field_name)
                    ));
                }
            }
        }

        // 构建 CREATE TABLE 语句
        let mut sql = format!(
            "CREATE TABLE {} (\n{}",
            Self::escape_identifier(database, &table_name),
            column_defs.join(",\n")
        );

        if !constraints.is_empty() {
            sql.push_str(",\n");
            sql.push_str(&constraints.join(",\n"));
        }

        // 添加表注释（MySQL 在 CREATE TABLE 语句中，必须在 ); 之前）
        if let Some(ref comment) = table_comment {
            match database {
                "mysql" => {
                    // MySQL: COMMENT 'comment text' 必须在 ); 之前
                    let escaped_comment = comment.replace('\'', "''");
                    sql.push_str(&format!("\n) COMMENT '{}';", escaped_comment));
                }
                _ => {
                    // PostgreSQL 和 SQLite 的表注释在 CREATE TABLE 之后单独处理
                    sql.push_str("\n);");
                }
            }
        } else {
            sql.push_str("\n);");
        }

        // 生成单字段普通索引（在 CREATE TABLE 之后）
        let has_single_indexes = !indexes.is_empty();
        if has_single_indexes {
            sql.push_str("\n\n");
            for (index_name, field_name) in indexes {
                sql.push_str(&format!(
                    "CREATE INDEX {} ON {} ({});\n",
                    Self::escape_identifier(database, &index_name),
                    Self::escape_identifier(database, &table_name),
                    Self::escape_identifier(database, &field_name)
                ));
            }
        }

        // 生成联合索引（在 CREATE TABLE 之后，按顺序排序）
        let has_composite_indexes = !composite_indexes.is_empty();
        if has_composite_indexes {
            if !has_single_indexes {
                sql.push_str("\n\n");
            }
            for (index_name, mut fields_with_order) in composite_indexes {
                // 按顺序排序
                fields_with_order.sort_by_key(|(_, order)| *order);
                let fields_str = fields_with_order
                    .iter()
                    .map(|(f, _)| Self::escape_identifier(database, f))
                    .collect::<Vec<_>>()
                    .join(", ");
                sql.push_str(&format!(
                    "CREATE INDEX {} ON {} ({});\n",
                    Self::escape_identifier(database, &index_name),
                    Self::escape_identifier(database, &table_name),
                    fields_str
                ));
            }
        }

        // 生成 PostgreSQL 的字段注释（在 CREATE TABLE 之后）
        let has_postgres_comments = !postgres_comments.is_empty();
        if database == "postgres" && has_postgres_comments {
            if !has_single_indexes && !has_composite_indexes {
                sql.push_str("\n\n");
            } else {
                sql.push_str("\n\n");
            }
            for (field_name, comment) in postgres_comments {
                // PostgreSQL: COMMENT ON COLUMN table.column IS 'comment text';
                // 需要转义单引号
                let escaped_comment = comment.replace('\'', "''");
                sql.push_str(&format!(
                    "COMMENT ON COLUMN {}.{} IS '{}';\n",
                    Self::escape_identifier(database, &table_name),
                    Self::escape_identifier(database, &field_name),
                    escaped_comment
                ));
            }
        }

        // 生成 PostgreSQL 的表注释（在 CREATE TABLE 之后）
        if database == "postgres" {
            if let Some(ref comment) = table_comment {
                // 判断是否需要添加换行
                let needs_newline = has_single_indexes || has_composite_indexes || has_postgres_comments;
                if needs_newline {
                    sql.push_str("\n\n");
                } else {
                    sql.push_str("\n\n");
                }
                // PostgreSQL: COMMENT ON TABLE table IS 'comment text';
                let escaped_comment = comment.replace('\'', "''");
                sql.push_str(&format!(
                    "COMMENT ON TABLE {} IS '{}';\n",
                    Self::escape_identifier(database, &table_name),
                    escaped_comment
                ));
            }
        }

        // 生成 SQLite 的表注释（在 CREATE TABLE 之后，使用 SQL 注释）
        if database == "sqlite" {
            if let Some(ref comment) = table_comment {
                // 判断是否需要添加换行
                let needs_newline = has_single_indexes || has_composite_indexes;
                if needs_newline {
                    sql.push_str("\n\n");
                } else {
                    sql.push_str("\n\n");
                }
                // SQLite: 使用 SQL 注释 -- comment text
                sql.push_str(&format!("-- 表注释: {}\n", comment));
            }
        }

        Ok(sql)
    }

    /// 解析 #[model(...)] 属性
    fn parse_model_attr(attr: &syn::Attribute) -> Result<(String, String, Option<String>, Option<String>)> {
        let mut table_name = None;
        let mut pk_field = None;
        let mut soft_delete_field = None;
        let mut table_comment = None;

        if let syn::Meta::List(list) = &attr.meta {
            let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
            if let Ok(metas) = parser.parse2(list.tokens.clone().into()) {
                for meta in metas {
                    if let syn::Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("table") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(s),
                                ..
                            }) = nv.value
                            {
                                table_name = Some(s.value());
                            }
                        } else if nv.path.is_ident("pk") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(s),
                                ..
                            }) = nv.value
                            {
                                pk_field = Some(s.value());
                            }
                        } else if nv.path.is_ident("soft_delete") {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(s),
                                ..
                            }) = nv.value
                            {
                                soft_delete_field = Some(s.value());
                            }
                        } else if nv.path.is_ident("table_comment") || nv.path.is_ident("comment") {
                            // 支持 table_comment 或 comment 作为表注释
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(s),
                                ..
                            }) = nv.value
                            {
                                table_comment = Some(s.value());
                            }
                        }
                    }
                }
            }
        }

        let table_name = table_name.context("Missing 'table' attribute in #[model(...)]")?;
        let pk_field = pk_field.unwrap_or_else(|| "id".to_string());

        Ok((table_name, pk_field, soft_delete_field, table_comment))
    }

    /// 格式化默认值为 SQL 格式
    /// 
    /// 处理规则：
    /// 1. 函数调用（CURRENT_TIMESTAMP, NOW()）直接使用
    /// 2. 空字符串使用 ''
    /// 3. 布尔类型根据数据库类型格式化：
    ///    - PostgreSQL BOOLEAN: TRUE/FALSE
    ///    - MySQL TINYINT(1): 0/1
    ///    - SQLite INTEGER (bool): 0/1
    /// 4. 数字直接使用
    /// 5. 字符串添加引号
    fn format_default_value(
        default: &str,
        rust_type: &Type,
        sql_type: &str,
        database: &str,
    ) -> Result<String> {
        // 1. 函数调用直接使用
        if default.starts_with("CURRENT_") || default.starts_with("NOW()") {
            return Ok(default.to_string());
        }

        // 2. 空字符串
        if default == "\"\"" || default == "''" || default.is_empty() {
            return Ok("''".to_string());
        }

        // 3. 布尔类型处理
        let sql_type_upper = sql_type.to_uppercase();
        let is_bool_rust_type = Self::is_bool_type(rust_type);
        let is_bool_sql_type = sql_type_upper == "BOOLEAN"
            || sql_type_upper == "TINYINT(1)"
            || (sql_type_upper.starts_with("TINYINT") && sql_type_upper.contains("(1)"))
            || sql_type_upper == "BIT(1)"
            || (sql_type_upper.starts_with("BIT") && sql_type_upper.contains("(1)"))
            || (database == "sqlite" && sql_type_upper == "INTEGER" && is_bool_rust_type);

        if is_bool_rust_type || is_bool_sql_type {
            return Ok(Self::format_boolean_default(default, sql_type, database));
        }

        // 4. 数字直接使用
        if default.parse::<i64>().is_ok() || default.parse::<f64>().is_ok() {
            return Ok(default.to_string());
        }

        // 5. 字符串处理
        if (default.starts_with('"') && default.ends_with('"'))
            || (default.starts_with('\'') && default.ends_with('\''))
        {
            Ok(default.to_string())
        } else {
            // 转义单引号
            let escaped = default.replace('\'', "''");
            Ok(format!("'{}'", escaped))
        }
    }

    /// 格式化布尔类型默认值
    /// 
    /// 支持的输入格式：
    /// - "0", "1"
    /// - "false", "true", "FALSE", "TRUE"
    /// - "b'0'", "b'1'" (PostgreSQL/MySQL 位字符串)
    /// - "b\'0\'", "b\'1\'" (转义的位字符串)
    /// 
    /// 输出格式：
    /// - PostgreSQL BOOLEAN: "FALSE" 或 "TRUE"
    /// - MySQL TINYINT(1): "0" 或 "1"
    /// - MySQL BIT(1): "b'0'" 或 "b'1'" (MySQL BIT 类型使用位字符串格式)
    /// - SQLite INTEGER: "0" 或 "1"
    fn format_boolean_default(default: &str, sql_type: &str, database: &str) -> String {
        // 移除转义字符
        let unescaped = default.replace("\\'", "'").replace("\\\"", "\"");

        // 处理位字符串字面量 b'0' 或 b'1'
        let cleaned = if unescaped.starts_with("b'") && unescaped.ends_with('\'') && unescaped.len() >= 4 {
            // 提取 b'0' 或 b'1' 中的数字部分
            unescaped[2..unescaped.len() - 1].to_string()
        } else {
            unescaped
        };

        // 转换为标准格式
        let bool_value = match cleaned.as_str() {
            "0" | "false" | "FALSE" => false,
            "1" | "true" | "TRUE" => true,
            _ => {
                // 尝试解析为数字
                if let Ok(num) = cleaned.parse::<i64>() {
                    num != 0
                } else {
                    // 尝试解析为布尔字符串
                    let upper = cleaned.to_uppercase();
                    if upper == "TRUE" {
                        true
                    } else if upper == "FALSE" {
                        false
                    } else {
                        eprintln!("Warning: Cannot parse boolean default value '{}', using false", default);
                        false
                    }
                }
            }
        };

        // 根据数据库类型和 SQL 类型格式化输出
        let sql_type_upper = sql_type.to_uppercase();
        if sql_type_upper == "BOOLEAN" && database == "postgres" {
            // PostgreSQL BOOLEAN 使用 TRUE/FALSE
            if bool_value {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        } else if (sql_type_upper == "BIT(1)" || (sql_type_upper.starts_with("BIT") && sql_type_upper.contains("(1)"))) && database == "mysql" {
            // MySQL BIT(1) 使用位字符串格式 b'0' 或 b'1'
            if bool_value {
                "b'1'".to_string()
            } else {
                "b'0'".to_string()
            }
        } else {
            // MySQL TINYINT(1) 和 SQLite INTEGER 使用数字 0/1
            if bool_value {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
    }

    /// 判断 Rust 类型是否为 bool（包括 Option<bool>）
    fn is_bool_type(ty: &Type) -> bool {
        match ty {
            Type::Path(TypePath { path, .. }) => {
                // 检查是否为 bool
                if let Some(segment) = path.segments.last() {
                    if segment.ident == "bool" {
                        return true;
                    }
                }
                // 检查是否为 Option<bool>
                if let Some(segment) = path.segments.first() {
                    if segment.ident == "Option" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                if let Type::Path(TypePath { path: inner_path, .. }) = inner_ty {
                                    if let Some(inner_segment) = inner_path.segments.last() {
                                        if inner_segment.ident == "bool" {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// 判断是否是 Option<T> 类型
    fn is_option_type(ty: &Type) -> bool {
        if let Type::Path(TypePath { path, .. }) = ty {
            if let Some(segment) = path.segments.last() {
                if segment.ident == "Option" {
                    return true;
                }
            }
        }
        false
    }

    /// Rust 类型转换为 SQL 类型（时间类型统一使用带时区的）
    fn rust_type_to_sql(ty: &Type, database: &str) -> Result<String> {
        // 如果是 Option<T>，提取内部类型
        if let Type::Path(TypePath { path, .. }) = ty {
            if let Some(segment) = path.segments.last() {
                if segment.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                            return Self::rust_type_to_sql(inner, database);
                        }
                    }
                }
            }
        }
        
        let inner_ty = ty;

        let type_str = quote::quote!(#inner_ty).to_string();
        let type_str_clean = type_str.replace(" ", "");

        let sql_type = match type_str_clean.as_str() {
            // 整数类型
            "i64" => match database {
                "mysql" => "BIGINT",
                "postgres" => "BIGINT",
                "sqlite" => "INTEGER",
                _ => "BIGINT",
            },
            "i32" => match database {
                "mysql" => "INT",
                "postgres" => "INTEGER",
                "sqlite" => "INTEGER",
                _ => "INT",
            },
            "i16" => match database {
                "mysql" => "TINYINT",
                "postgres" => "SMALLINT",
                "sqlite" => "INTEGER",
                _ => "SMALLINT",
            },
            "u64" => match database {
                "mysql" => "BIGINT UNSIGNED",
                "postgres" => "BIGINT",
                "sqlite" => "INTEGER",
                _ => "BIGINT UNSIGNED",
            },
            "u32" => match database {
                "mysql" => "INT UNSIGNED",
                "postgres" => "INTEGER",
                "sqlite" => "INTEGER",
                _ => "INT UNSIGNED",
            },
            "u16" => match database {
                "mysql" => "SMALLINT UNSIGNED",
                "postgres" => "SMALLINT",
                "sqlite" => "INTEGER",
                _ => "SMALLINT UNSIGNED",
            },
            // 字符串类型
            "String" => match database {
                "mysql" => "VARCHAR(255)",
                "postgres" => "VARCHAR(255)",
                "sqlite" => "TEXT",
                _ => "VARCHAR(255)",
            },
            // 数值类型
            "f64" => match database {
                "mysql" => "DOUBLE",
                "postgres" => "DOUBLE PRECISION",
                "sqlite" => "REAL",
                _ => "DOUBLE",
            },
            "f32" => match database {
                "mysql" => "FLOAT",
                "postgres" => "REAL",
                "sqlite" => "REAL",
                _ => "FLOAT",
            },
            // 布尔类型
            "bool" => match database {
                "mysql" => "TINYINT(1)",
                "postgres" => "BOOLEAN",
                "sqlite" => "INTEGER",
                _ => "BOOLEAN",
            },
            // 日期时间类型 - 统一使用带时区的
            _ if type_str_clean.contains("chrono::DateTime<chrono::Utc>") 
                || type_str_clean.contains("DateTime<Utc>") => {
                match database {
                    "mysql" => "TIMESTAMP(3)", // MySQL 的 TIMESTAMP 是带时区的，使用 (3) 以兼容更多版本
                    "postgres" => "TIMESTAMP WITH TIME ZONE",
                    "sqlite" => "TEXT", // SQLite 使用 TEXT 存储时间戳
                    _ => "TIMESTAMP WITH TIME ZONE",
                }
            }
            _ if type_str_clean.contains("chrono::NaiveDateTime") 
                || type_str_clean.contains("NaiveDateTime") => {
                // NaiveDateTime 也转换为带时区的（根据用户要求）
                match database {
                    "mysql" => "TIMESTAMP(3)", // MySQL 的 TIMESTAMP 是带时区的，使用 (3) 以兼容更多版本
                    "postgres" => "TIMESTAMP WITH TIME ZONE",
                    "sqlite" => "TEXT",
                    _ => "TIMESTAMP WITH TIME ZONE",
                }
            }
            _ if type_str_clean.contains("chrono::NaiveDate") 
                || type_str_clean.contains("NaiveDate") => {
                match database {
                    "mysql" => "DATE",
                    "postgres" => "DATE",
                    "sqlite" => "TEXT",
                    _ => "DATE",
                }
            }
            _ if type_str_clean.contains("chrono::NaiveTime") 
                || type_str_clean.contains("NaiveTime") => {
                match database {
                    "mysql" => "TIME(3)", // 使用 (3) 以兼容更多 MySQL 版本
                    "postgres" => "TIME WITH TIME ZONE",
                    "sqlite" => "TEXT",
                    _ => "TIME WITH TIME ZONE",
                }
            }
            // JSON 类型
            _ if type_str_clean.contains("serde_json::Value") 
                || type_str_clean.contains("Value") => {
                match database {
                    "mysql" => "JSON",
                    "postgres" => "JSONB",
                    "sqlite" => "TEXT",
                    _ => "JSON",
                }
            }
            // UUID 类型
            _ if type_str_clean.contains("uuid::Uuid") 
                || type_str_clean.contains("Uuid") => {
                match database {
                    "mysql" => "CHAR(36)",
                    "postgres" => "UUID",
                    "sqlite" => "TEXT",
                    _ => "UUID",
                }
            }
            // 二进制类型
            _ if type_str_clean.contains("Vec<u8>") => {
                match database {
                    "mysql" => "BLOB",
                    "postgres" => "BYTEA",
                    "sqlite" => "BLOB",
                    _ => "BLOB",
                }
            }
            // 默认返回 VARCHAR(255)
            _ => {
                eprintln!("Warning: Unknown type '{}', using VARCHAR(255) as default", type_str_clean);
                match database {
                    "mysql" => "VARCHAR(255)",
                    "postgres" => "VARCHAR(255)",
                    "sqlite" => "TEXT",
                    _ => "VARCHAR(255)",
                }
            }
        };

        Ok(sql_type.to_string())
    }

    /// 解析字段的 column 属性
    fn parse_column_attr(attrs: &[syn::Attribute]) -> Result<ColumnMeta> {
        let mut meta = ColumnMeta::default();
        
        for attr in attrs {
            if attr.path().is_ident("column") {
                if let syn::Meta::List(list) = &attr.meta {
                    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
                    if let Ok(metas) = parser.parse2(list.tokens.clone().into()) {
                        for meta_item in metas {
                            match meta_item {
                                syn::Meta::Path(path) => {
                                    if path.is_ident("index") {
                                        // index 作为路径，表示使用默认索引名称
                                        meta.index = Some(String::new());
                                    } else if path.is_ident("unique") {
                                        meta.unique = true;
                                    } else if path.is_ident("not_null") {
                                        meta.not_null = true;
                                    } else if path.is_ident("auto_increment") {
                                        meta.auto_increment = true;
                                    } else if path.is_ident("primary_key") || path.is_ident("pk") {
                                        meta.primary_key = true;
                                    } else if path.is_ident("soft_delete") {
                                        meta.soft_delete = true;
                                    }
                                }
                                syn::Meta::NameValue(nv) => {
                                    if nv.path.is_ident("default") {
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = nv.value
                                        {
                                            meta.default = Some(s.value());
                                        }
                                    } else if nv.path.is_ident("length") {
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Int(i),
                                            ..
                                        }) = nv.value
                                        {
                                            if let Ok(len) = i.base10_parse::<u32>() {
                                                meta.length = Some(len);
                                            }
                                        }
                                    } else if nv.path.is_ident("index") {
                                        // index = "index_name" 用于指定索引名称，空字符串表示使用默认名称
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = nv.value
                                        {
                                            meta.index = Some(s.value());
                                        }
                                    } else if nv.path.is_ident("combine_index") {
                                        // combine_index = "index_name" 或 "index_name:order" 用于指定联合索引名称和顺序
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = nv.value
                                        {
                                            let value = s.value();
                                            // 支持两种格式：
                                            // 1. "index_name" - 只指定名称，顺序按字段在结构体中的位置
                                            // 2. "index_name:order" - 指定名称和顺序
                                            if let Some((name, order_str)) = value.split_once(':') {
                                                if let Ok(order) = order_str.trim().parse::<i32>() {
                                                    meta.combine_index = Some((name.to_string(), order));
                                                } else {
                                                    // 如果解析顺序失败，只使用名称，顺序为 0
                                                    meta.combine_index = Some((value, 0));
                                                }
                                            } else {
                                                // 没有指定顺序，使用字段在结构体中的位置（稍后设置）
                                                meta.combine_index = Some((value, i32::MAX));
                                            }
                                        }
                                    } else if nv.path.is_ident("comment") {
                                        // comment = "comment text" 用于指定字段注释
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = nv.value
                                        {
                                            meta.comment = Some(s.value());
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        
        Ok(meta)
    }

    /// 转义标识符（根据数据库类型）
    fn escape_identifier(database: &str, name: &str) -> String {
        match database {
            "mysql" => format!("`{}`", name),
            "postgres" => format!("\"{}\"", name),
            "sqlite" => format!("\"{}\"", name),
            _ => name.to_string(),
        }
    }
}

/// 字段的 column 属性元数据
#[derive(Default)]
struct ColumnMeta {
    index: Option<String>, // None 表示不创建索引，Some("") 表示使用默认名称，Some(name) 表示使用指定名称
    combine_index: Option<(String, i32)>, // None 表示不创建联合索引，Some((name, order)) 表示加入名为 name 的联合索引，order 指定顺序
    unique: bool,
    not_null: bool,
    default: Option<String>,
    length: Option<u32>,
    auto_increment: bool,
    primary_key: bool,
    soft_delete: bool,
    comment: Option<String>, // 字段注释
}

