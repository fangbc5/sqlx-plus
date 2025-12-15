use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, Data, DataStruct, DeriveInput, Fields, Meta};

/// 生成 Model trait 的实现
///
/// 自动生成 `TABLE`、`PK` 和可选的 `SOFT_DELETE_FIELD` 常量
///
/// 使用示例：
/// ```ignore
/// // 物理删除模式（默认）
/// #[derive(ModelMeta)]
/// #[model(table = "users", pk = "id")]
/// struct User {
///     id: i64,
///     name: String,
/// }
///
/// // 逻辑删除模式
/// #[derive(ModelMeta)]
/// #[model(table = "users", pk = "id", soft_delete = "is_deleted")]
/// struct UserWithSoftDelete {
///     id: i64,
///     name: String,
///     is_deleted: i32, // 逻辑删除字段：0=未删除，1=已删除
/// }
/// ```
#[proc_macro_derive(ModelMeta, attributes(model))]
pub fn derive_model_meta(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // 解析属性
    let mut table_name = None;
    let mut pk_field = None;
    let mut soft_delete_field = None;

    for attr in &input.attrs {
        if attr.path().is_ident("model") {
            // 在 syn 2.0 中，使用 meta() 方法获取元数据
            if let syn::Meta::List(list) = &attr.meta {
                // 解析列表中的每个 Meta::NameValue
                let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
                if let Ok(metas) = parser.parse2(list.tokens.clone()) {
                    for meta in metas {
                        if let Meta::NameValue(nv) = meta {
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
                            }
                        }
                    }
                }
            } else if let syn::Meta::NameValue(nv) = &attr.meta {
                // 单个 NameValue 的情况
                if nv.path.is_ident("table") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        table_name = Some(s.value());
                    }
                } else if nv.path.is_ident("pk") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        pk_field = Some(s.value());
                    }
                } else if nv.path.is_ident("soft_delete") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        soft_delete_field = Some(s.value());
                    }
                }
            }
        }
    }

    // 如果没有指定表名，使用结构体名称的小写蛇形命名方式
    let table = table_name.unwrap_or_else(|| {
        let s = name.to_string();
        // 将 PascalCase 转换为 snake_case
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        }
        result
    });

    // 如果没有指定主键，默认使用 "id"
    let pk = pk_field.unwrap_or_else(|| "id".to_string());

    // 生成实现代码
    let expanded = if let Some(soft_delete) = soft_delete_field {
        // 如果指定了逻辑删除字段，生成包含 SOFT_DELETE_FIELD 的实现
        let soft_delete_lit = syn::LitStr::new(&soft_delete, proc_macro2::Span::call_site());
        quote! {
            impl sqlx_plus_core::Model for #name {
                const TABLE: &'static str = #table;
                const PK: &'static str = #pk;
                const SOFT_DELETE_FIELD: Option<&'static str> = Some(#soft_delete_lit);
            }
        }
    } else {
        // 如果没有指定逻辑删除字段，SOFT_DELETE_FIELD 为 None
        quote! {
            impl sqlx_plus_core::Model for #name {
                const TABLE: &'static str = #table;
                const PK: &'static str = #pk;
                const SOFT_DELETE_FIELD: Option<&'static str> = None;
            }
        }
    };

    TokenStream::from(expanded)
}

/// 生成 CRUD trait 的实现
///
/// 自动生成 insert 和 update 方法的实现
///
/// 使用示例：
/// ```ignore
/// // 物理删除模式
/// #[derive(CRUD, FromRow, ModelMeta)]
/// #[model(table = "users", pk = "id")]
/// struct User {
///     id: i64,
///     name: String,
///     email: String,
/// }
///
/// // 逻辑删除模式
/// #[derive(CRUD, FromRow, ModelMeta)]
/// #[model(table = "users", pk = "id", soft_delete = "is_deleted")]
/// struct UserWithSoftDelete {
///     id: i64,
///     name: String,
///     email: String,
///     is_deleted: i32, // 逻辑删除字段
/// }
/// ```
#[proc_macro_derive(CRUD, attributes(model, skip))]
pub fn derive_crud(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // 解析 #[model(pk = "...")]，获取主键字段名，默认 "id"
    let mut pk_field = None;
    for attr in &input.attrs {
        if attr.path().is_ident("model") {
            if let syn::Meta::List(list) = &attr.meta {
                let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
                if let Ok(metas) = parser.parse2(list.tokens.clone()) {
                    for meta in metas {
                        if let Meta::NameValue(nv) = meta {
                            if nv.path.is_ident("pk") {
                                if let syn::Expr::Lit(syn::ExprLit {
                                    lit: syn::Lit::Str(s),
                                    ..
                                }) = nv.value
                                {
                                    pk_field = Some(s.value());
                                }
                            }
                        }
                    }
                }
            } else if let syn::Meta::NameValue(nv) = &attr.meta {
                if nv.path.is_ident("pk") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        pk_field = Some(s.value());
                    }
                }
            }
        }
    }
    // 如果没有指定主键，默认使用 "id"
    let pk = pk_field.unwrap_or_else(|| "id".to_string());

    // 获取字段列表（必须是具名字段的结构体）
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => {
            return syn::Error::new_spanned(
                name,
                "CRUD derive only supports structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    // 收集字段信息
    // - field_names / field_columns: 所有字段（包括主键）
    // - insert_field_*: 用于 INSERT（排除主键）
    // - update_field_*: 用于 UPDATE SET 子句（排除主键）
    let mut field_names = Vec::new();
    let mut field_columns = Vec::new();
    let mut skip_fields = Vec::new();
    let mut insert_field_names = Vec::new();
    let mut insert_field_columns = Vec::new();
    let mut update_field_names = Vec::new();
    // 主键字段的 Ident，用于绑定 WHERE pk = ?
    let mut pk_ident_opt: Option<&syn::Ident> = None;

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        // 检查是否有 skip 属性
        let mut skip = false;
        for attr in &field.attrs {
            if attr.path().is_ident("skip") || attr.path().is_ident("model") {
                skip = true;
                break;
            }
        }

        if !skip {
            field_names.push(field_name);
            field_columns.push(field_name_str.clone());
            if field_name_str == pk {
                // 记录主键字段
                pk_ident_opt = Some(field_name);
            } else {
                // 非主键字段用于 INSERT / UPDATE
                insert_field_names.push(field_name);
                insert_field_columns.push(field_name_str.clone());
                update_field_names.push(field_name);
            }
        } else {
            skip_fields.push(field_name_str);
        }
    }

    // 编译期确保主键字段存在
    let pk_ident = pk_ident_opt.expect("Primary key field not found in struct");

    // 生成 insert SQL（排除主键列，依赖数据库自增 / identity）
    let insert_fields: Vec<String> = insert_field_columns.clone();
    let insert_placeholders: Vec<String> =
        (0..insert_fields.len()).map(|_| "?".to_string()).collect();
    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        format!("{{TABLE}}"), // 占位符，运行时替换
        insert_fields.join(", "),
        insert_placeholders.join(", ")
    );

    // 生成 update SQL（排除主键列，只更新非主键列）
    let update_fields: Vec<String> = field_columns
        .iter()
        .filter(|f| *f != &pk)
        .cloned()
        .collect();
    let update_sql = if !update_fields.is_empty() {
        format!(
            "UPDATE {} SET {} WHERE {} = ?",
            format!("{{TABLE}}"),
            update_fields
                .iter()
                .map(|f| format!("{} = ?", f))
                .collect::<Vec<_>>()
                .join(", "),
            format!("{{PK}}")
        )
    } else {
        String::new()
    };

    // 生成实现代码
    let expanded = quote! {
        #[async_trait::async_trait]
        impl sqlx_plus_core::Crud for #name {
            async fn insert(&self, pool: &sqlx_plus_core::DbPool) -> sqlx_plus_core::db_pool::Result<sqlx_plus_core::crud::Id> {
                use sqlx_plus_core::Model;
                use sqlx_plus_core::utils::escape_identifier;
                let table = Self::TABLE;
                let driver = pool.driver();
                let escaped_table = escape_identifier(driver, table);
                let sql = #insert_sql.replace("{TABLE}", &escaped_table);
                let sql = pool.convert_sql(&sql);

                match pool.driver() {
                    sqlx_plus_core::db_pool::DbDriver::MySql => {
                        let pool_ref = pool.mysql_pool().ok_or(sqlx_plus_core::db_pool::DbPoolError::NoPoolAvailable)?;
                        let result = sqlx::query(&sql)
                            #( .bind(&self.#insert_field_names) )*
                            .execute(pool_ref)
                            .await?;
                        Ok(result.last_insert_id() as i64)
                    }
                    sqlx_plus_core::db_pool::DbDriver::Postgres => {
                        let pool_ref = pool.pg_pool().ok_or(sqlx_plus_core::db_pool::DbPoolError::NoPoolAvailable)?;
                        let pk = Self::PK;
                        use sqlx_plus_core::utils::escape_identifier;
                        let escaped_pk = escape_identifier(sqlx_plus_core::db_pool::DbDriver::Postgres, pk);
                        // 为 PostgreSQL 添加 RETURNING 子句
                        let sql_with_returning = format!("{} RETURNING {}", sql, escaped_pk);
                        let id: i64 = sqlx::query_scalar(&sql_with_returning)
                            #( .bind(&self.#insert_field_names) )*
                            .fetch_one(pool_ref)
                            .await?;
                        Ok(id)
                    }
                    sqlx_plus_core::db_pool::DbDriver::Sqlite => {
                        let pool_ref = pool.sqlite_pool().ok_or(sqlx_plus_core::db_pool::DbPoolError::NoPoolAvailable)?;
                        let result = sqlx::query(&sql)
                            #( .bind(&self.#insert_field_names) )*
                            .execute(pool_ref)
                            .await?;
                        Ok(result.last_insert_rowid() as i64)
                    }
                }
            }

            async fn update(&self, pool: &sqlx_plus_core::DbPool) -> sqlx_plus_core::db_pool::Result<()> {
                use sqlx_plus_core::Model;
                use sqlx_plus_core::utils::escape_identifier;
                let table = Self::TABLE;
                let pk = Self::PK;
                let driver = pool.driver();
                let escaped_table = escape_identifier(driver, table);
                let escaped_pk = escape_identifier(driver, pk);
                let sql = #update_sql.replace("{TABLE}", &escaped_table).replace("{PK}", &escaped_pk);
                let sql = pool.convert_sql(&sql);

                match pool.driver() {
                    sqlx_plus_core::db_pool::DbDriver::MySql => {
                        let pool_ref = pool.mysql_pool().ok_or(sqlx_plus_core::db_pool::DbPoolError::NoPoolAvailable)?;
                        sqlx::query(&sql)
                            #( .bind(&self.#update_field_names) )*
                            .bind(&self.#pk_ident)
                            .execute(pool_ref)
                            .await?;
                    }
                    sqlx_plus_core::db_pool::DbDriver::Postgres => {
                        let pool_ref = pool.pg_pool().ok_or(sqlx_plus_core::db_pool::DbPoolError::NoPoolAvailable)?;
                        sqlx::query(&sql)
                            #( .bind(&self.#update_field_names) )*
                            .bind(&self.#pk_ident)
                            .execute(pool_ref)
                            .await?;
                    }
                    sqlx_plus_core::db_pool::DbDriver::Sqlite => {
                        let pool_ref = pool.sqlite_pool().ok_or(sqlx_plus_core::db_pool::DbPoolError::NoPoolAvailable)?;
                        sqlx::query(&sql)
                            #( .bind(&self.#update_field_names) )*
                            .bind(&self.#pk_ident)
                            .execute(pool_ref)
                            .await?;
                    }
                }
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
