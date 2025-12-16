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
            impl sqlxplus::Model for #name {
                const TABLE: &'static str = #table;
                const PK: &'static str = #pk;
                const SOFT_DELETE_FIELD: Option<&'static str> = Some(#soft_delete_lit);
            }
        }
    } else {
        // 如果没有指定逻辑删除字段，SOFT_DELETE_FIELD 为 None
        quote! {
            impl sqlxplus::Model for #name {
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
    // - pk_ident: 主键字段 Ident
    // - insert_*/update_*: 非主键字段（INSERT / UPDATE 使用）
    let mut pk_ident_opt: Option<&syn::Ident> = None;

    // INSERT 使用的字段（非主键）
    let mut insert_normal_field_names: Vec<&syn::Ident> = Vec::new();
    let mut insert_normal_field_columns: Vec<syn::LitStr> = Vec::new();
    let mut insert_option_field_names: Vec<&syn::Ident> = Vec::new();
    let mut insert_option_field_columns: Vec<syn::LitStr> = Vec::new();

    // UPDATE 使用的字段（非主键）
    let mut update_normal_field_names: Vec<&syn::Ident> = Vec::new();
    let mut update_normal_field_columns: Vec<syn::LitStr> = Vec::new();
    let mut update_option_field_names: Vec<&syn::Ident> = Vec::new();
    let mut update_option_field_columns: Vec<syn::LitStr> = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        // 检查属性：skip / model
        let mut skip = false;
        for attr in &field.attrs {
            if attr.path().is_ident("skip") || attr.path().is_ident("model") {
                skip = true;
                break;
            }
        }

        if !skip {
            if field_name_str == pk {
                // 记录主键字段
                pk_ident_opt = Some(field_name);
            } else {
                // 非主键字段用于 INSERT / UPDATE
                let is_opt = is_option_type(&field.ty);
                let col_lit = syn::LitStr::new(&field_name_str, proc_macro2::Span::call_site());

                if is_opt {
                    insert_option_field_names.push(field_name);
                    insert_option_field_columns.push(col_lit.clone());

                    update_option_field_names.push(field_name);
                    update_option_field_columns.push(col_lit);
                } else {
                    insert_normal_field_names.push(field_name);
                    insert_normal_field_columns.push(col_lit.clone());

                    update_normal_field_names.push(field_name);
                    update_normal_field_columns.push(col_lit);
                }
            }
        }
    }

    // 编译期确保主键字段存在
    let pk_ident = pk_ident_opt.expect("Primary key field not found in struct");

    // 生成实现代码
    let expanded = quote! {
        #[async_trait::async_trait]
        impl sqlxplus::Crud for #name {
            async fn insert(&self, pool: &sqlxplus::DbPool) -> sqlxplus::db_pool::Result<sqlxplus::crud::Id> {
                use sqlxplus::Model;
                use sqlxplus::utils::escape_identifier;
                let table = Self::TABLE;
                let driver = pool.driver();
                let escaped_table = escape_identifier(driver, table);

                // 构建列名和占位符：
                // - 非 Option 字段：始终参与 INSERT
                // - Option 字段：仅在 Some 时参与 INSERT（None 则跳过，让数据库用默认值）
                let mut columns: Vec<&str> = Vec::new();
                let mut placeholders: Vec<&str> = Vec::new();

                // 非 Option 字段：始终参与 INSERT
                #(
                    columns.push(#insert_normal_field_columns);
                    placeholders.push("?");
                )*

                // Option 字段：仅当为 Some 时参与 INSERT
                #(
                    if self.#insert_option_field_names.is_some() {
                        columns.push(#insert_option_field_columns);
                        placeholders.push("?");
                    }
                )*

                let raw_sql = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    escaped_table,
                    columns.join(", "),
                    placeholders.join(", ")
                );
                let sql = pool.convert_sql(&raw_sql);

                match pool.driver() {
                    sqlxplus::db_pool::DbDriver::MySql => {
                        let pool_ref = pool.mysql_pool().ok_or(sqlxplus::db_pool::DbPoolError::NoPoolAvailable)?;
                        let mut query = sqlx::query(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#insert_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if self.#insert_option_field_names.is_some() {
                                query = query.bind(&self.#insert_option_field_names);
                            }
                        )*
                        let result = query
                            .execute(pool_ref)
                            .await?;
                        Ok(result.last_insert_id() as i64)
                    }
                    sqlxplus::db_pool::DbDriver::Postgres => {
                        let pool_ref = pool.pg_pool().ok_or(sqlxplus::db_pool::DbPoolError::NoPoolAvailable)?;
                        let pk = Self::PK;
                        use sqlxplus::utils::escape_identifier;
                        let escaped_pk = escape_identifier(sqlxplus::db_pool::DbDriver::Postgres, pk);
                        // 为 PostgreSQL 添加 RETURNING 子句
                        let sql_with_returning = format!("{} RETURNING {}", sql, escaped_pk);
                        let mut query = sqlx::query_scalar::<_, i64>(&sql_with_returning);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#insert_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if self.#insert_option_field_names.is_some() {
                                query = query.bind(&self.#insert_option_field_names);
                            }
                        )*
                        let id: i64 = query
                            .fetch_one(pool_ref)
                            .await?;
                        Ok(id)
                    }
                    sqlxplus::db_pool::DbDriver::Sqlite => {
                        let pool_ref = pool.sqlite_pool().ok_or(sqlxplus::db_pool::DbPoolError::NoPoolAvailable)?;
                        let mut query = sqlx::query(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#insert_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if self.#insert_option_field_names.is_some() {
                                query = query.bind(&self.#insert_option_field_names);
                            }
                        )*
                        let result = query
                            .execute(pool_ref)
                            .await?;
                        Ok(result.last_insert_rowid() as i64)
                    }
                }
            }

            async fn update(&self, pool: &sqlxplus::DbPool) -> sqlxplus::db_pool::Result<()> {
                use sqlxplus::Model;
                use sqlxplus::utils::escape_identifier;
                let table = Self::TABLE;
                let pk = Self::PK;
                let driver = pool.driver();
                let escaped_table = escape_identifier(driver, table);
                let escaped_pk = escape_identifier(driver, pk);

                // 构建 UPDATE SET 子句（Patch 语义）：
                // - 非 Option 字段：始终参与更新
                // - Option 字段：仅当为 Some 时参与更新
                let mut set_parts: Vec<String> = Vec::new();

                // 非 Option 字段
                #(
                    set_parts.push(format!("{} = ?", #update_normal_field_columns));
                )*

                // Option 字段
                #(
                    if self.#update_option_field_names.is_some() {
                        set_parts.push(format!("{} = ?", #update_option_field_columns));
                    }
                )*

                let raw_sql = if !set_parts.is_empty() {
                    format!(
                        "UPDATE {} SET {} WHERE {} = ?",
                        escaped_table,
                        set_parts.join(", "),
                        escaped_pk,
                    )
                } else {
                    // 没有需要更新的字段，直接返回 Ok(())
                    return Ok(());
                };

                let sql = pool.convert_sql(&raw_sql);

                match pool.driver() {
                    sqlxplus::db_pool::DbDriver::MySql => {
                        let pool_ref = pool.mysql_pool().ok_or(sqlxplus::db_pool::DbPoolError::NoPoolAvailable)?;
                        let mut query = sqlx::query(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#update_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if self.#update_option_field_names.is_some() {
                                query = query.bind(&self.#update_option_field_names);
                            }
                        )*
                        query
                            .bind(&self.#pk_ident)
                            .execute(pool_ref)
                            .await?;
                    }
                    sqlxplus::db_pool::DbDriver::Postgres => {
                        let pool_ref = pool.pg_pool().ok_or(sqlxplus::db_pool::DbPoolError::NoPoolAvailable)?;
                        let mut query = sqlx::query(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#update_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if self.#update_option_field_names.is_some() {
                                query = query.bind(&self.#update_option_field_names);
                            }
                        )*
                        query
                            .bind(&self.#pk_ident)
                            .execute(pool_ref)
                            .await?;
                    }
                    sqlxplus::db_pool::DbDriver::Sqlite => {
                        let pool_ref = pool.sqlite_pool().ok_or(sqlxplus::db_pool::DbPoolError::NoPoolAvailable)?;
                        let mut query = sqlx::query(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#update_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if self.#update_option_field_names.is_some() {
                                query = query.bind(&self.#update_option_field_names);
                            }
                        )*
                        query
                            .bind(&self.#pk_ident)
                            .execute(pool_ref)
                            .await?;
                    }
                }
                Ok(())
            }

            /// 更新记录（包含 Option 字段为 None 的重置）
            ///
            /// - 非 Option 字段：始终参与更新（col = ?）
            /// - Option 字段：
            ///   - Some(v)：col = ?
            ///   - None：col = DEFAULT（由数据库决定置空或使用默认值）
            async fn update_with_none(&self, pool: &sqlxplus::DbPool) -> sqlxplus::db_pool::Result<()> {
                use sqlxplus::Model;
                use sqlxplus::utils::escape_identifier;
                let table = Self::TABLE;
                let pk = Self::PK;
                let driver = pool.driver();
                let escaped_table = escape_identifier(driver, table);
                let escaped_pk = escape_identifier(driver, pk);

                // 构建 UPDATE SET 子句（Reset 语义）
                let mut set_parts: Vec<String> = Vec::new();

                // 非 Option 字段：始终更新为当前值
                #(
                    set_parts.push(format!("{} = ?", #update_normal_field_columns));
                )*

                // Option 字段：Some -> ?，None -> DEFAULT/NULL（根据驱动类型）
                // SQLite 不支持 DEFAULT，且不可空字段不能设置为 NULL，所以跳过 None 字段的更新
                match driver {
                    sqlxplus::db_pool::DbDriver::Sqlite => {
                        // SQLite: 仅更新 Some 值的字段，跳过 None 字段（因为 SQLite 不支持 DEFAULT）
                        #(
                            if self.#update_option_field_names.is_some() {
                                set_parts.push(format!("{} = ?", #update_option_field_columns));
                            }
                            // None 字段跳过，不包含在 SET 子句中
                        )*
                    }
                    _ => {
                        // MySQL/PostgreSQL: None -> DEFAULT
                        #(
                            if self.#update_option_field_names.is_some() {
                                set_parts.push(format!("{} = ?", #update_option_field_columns));
                            } else {
                                set_parts.push(format!("{} = DEFAULT", #update_option_field_columns));
                            }
                        )*
                    }
                }

                if set_parts.is_empty() {
                    return Ok(());
                }

                let raw_sql = format!(
                    "UPDATE {} SET {} WHERE {} = ?",
                    escaped_table,
                    set_parts.join(", "),
                    escaped_pk,
                );

                let sql = pool.convert_sql(&raw_sql);

                match pool.driver() {
                    sqlxplus::db_pool::DbDriver::MySql => {
                        let pool_ref = pool.mysql_pool().ok_or(sqlxplus::db_pool::DbPoolError::NoPoolAvailable)?;
                        let mut query = sqlx::query(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#update_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定（None 使用 DEFAULT）
                        #(
                            if self.#update_option_field_names.is_some() {
                                query = query.bind(&self.#update_option_field_names);
                            }
                        )*
                        query
                            .bind(&self.#pk_ident)
                            .execute(pool_ref)
                            .await?;
                    }
                    sqlxplus::db_pool::DbDriver::Postgres => {
                        let pool_ref = pool.pg_pool().ok_or(sqlxplus::db_pool::DbPoolError::NoPoolAvailable)?;
                        let mut query = sqlx::query(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#update_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if self.#update_option_field_names.is_some() {
                                query = query.bind(&self.#update_option_field_names);
                            }
                        )*
                        query
                            .bind(&self.#pk_ident)
                            .execute(pool_ref)
                            .await?;
                    }
                    sqlxplus::db_pool::DbDriver::Sqlite => {
                        let pool_ref = pool.sqlite_pool().ok_or(sqlxplus::db_pool::DbPoolError::NoPoolAvailable)?;
                        let mut query = sqlx::query(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#update_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if self.#update_option_field_names.is_some() {
                                query = query.bind(&self.#update_option_field_names);
                            }
                        )*
                        query
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

/// 判断字段类型是否为 Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    return args.args.len() == 1;
                }
            }
        }
    }
    false
}

