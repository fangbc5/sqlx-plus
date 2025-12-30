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
        // Trait 方法实现
        #[async_trait::async_trait]
        impl sqlxplus::Crud for #name {
            // 泛型版本的 insert
            async fn insert<'e, 'c: 'e, DB, E>(&self, executor: E) -> sqlxplus::Result<sqlxplus::crud::Id>
            where
                DB: sqlx::Database + sqlxplus::DatabaseInfo,
                for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
                E: sqlx::Executor<'c, Database = DB> + Send,
                i64: sqlx::Type<DB> + for<'r> sqlx::Decode<'r, DB>,
                usize: sqlx::ColumnIndex<DB::Row>,
                // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
                String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                i64: for<'b> sqlx::Encode<'b, DB>,
                i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<i64>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<i32>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<i16>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                chrono::DateTime<chrono::Utc>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<chrono::DateTime<chrono::Utc>>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                serde_json::Value: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<serde_json::Value>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
            {
                use sqlxplus::Model;
                use sqlxplus::DatabaseInfo;
                use sqlxplus::db_pool::DbDriver;
                let table = Self::TABLE;
                let escaped_table = DB::escape_identifier(table);

                // 构建列名和占位符
                let mut columns: Vec<&str> = Vec::new();
                let mut placeholders: Vec<String> = Vec::new();
                let mut placeholder_index = 0;

                // 非 Option 字段：始终参与 INSERT
                #(
                    columns.push(#insert_normal_field_columns);
                    placeholders.push(DB::placeholder(placeholder_index));
                    placeholder_index += 1;
                )*

                // Option 字段：仅当为 Some 时参与 INSERT
                #(
                    if self.#insert_option_field_names.is_some() {
                        columns.push(#insert_option_field_columns);
                        placeholders.push(DB::placeholder(placeholder_index));
                        placeholder_index += 1;
                    }
                )*

                // 根据数据库类型构建 SQL
                let sql = match DB::get_driver() {
                    DbDriver::Postgres => {
                        let pk = Self::PK;
                        let escaped_pk = DB::escape_identifier(pk);
                        format!(
                            "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
                            escaped_table,
                            columns.join(", "),
                            placeholders.join(", "),
                            escaped_pk
                        )
                    }
                    _ => {
                        format!(
                            "INSERT INTO {} ({}) VALUES ({})",
                            escaped_table,
                            columns.join(", "),
                            placeholders.join(", ")
                        )
                    }
                };

                // 根据数据库类型执行查询
                match DB::get_driver() {
                    DbDriver::Postgres => {
                        let mut query = sqlx::query_scalar::<_, i64>(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#insert_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if let Some(ref val) = self.#insert_option_field_names {
                                query = query.bind(val);
                            }
                        )*
                        let id: i64 = query.fetch_one(executor).await?;
                        Ok(id)
                    }
                    DbDriver::MySql => {
                        let mut query = sqlx::query(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#insert_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if let Some(ref val) = self.#insert_option_field_names {
                                query = query.bind(val);
                            }
                        )*
                        let result = query.execute(executor).await?;
                        // 在泛型上下文中，我们需要使用 unsafe 转换来访问数据库特定的方法
                        // 这是安全的，因为我们已经通过 DB::get_driver() 确认了数据库类型
                        // 并且我们知道 DB = MySql，所以 result 的类型是 MySqlQueryResult
                        unsafe {
                            use sqlx::mysql::MySqlQueryResult;
                            let ptr: *const DB::QueryResult = &result;
                            let mysql_ptr = ptr as *const MySqlQueryResult;
                            Ok((*mysql_ptr).last_insert_id() as i64)
                        }
                    }
                    DbDriver::Sqlite => {
                        let mut query = sqlx::query(&sql);
                        // 非 Option 字段：始终绑定
                        #(
                            query = query.bind(&self.#insert_normal_field_names);
                        )*
                        // Option 字段：仅当为 Some 时绑定
                        #(
                            if let Some(ref val) = self.#insert_option_field_names {
                                query = query.bind(val);
                            }
                        )*
                        let result = query.execute(executor).await?;
                        // 在泛型上下文中，我们需要使用 unsafe 转换来访问数据库特定的方法
                        unsafe {
                            use sqlx::sqlite::SqliteQueryResult;
                            let ptr: *const DB::QueryResult = &result;
                            let sqlite_ptr = ptr as *const SqliteQueryResult;
                            Ok((*sqlite_ptr).last_insert_rowid() as i64)
                        }
                    }
                }
            }

            // 泛型版本的 update
            async fn update<'e, 'c: 'e, DB, E>(&self, executor: E) -> sqlxplus::Result<()>
            where
                DB: sqlx::Database + sqlxplus::DatabaseInfo,
                for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
                E: sqlx::Executor<'c, Database = DB> + Send,
                // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
                String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<i64>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<i32>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<i16>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                chrono::DateTime<chrono::Utc>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<chrono::DateTime<chrono::Utc>>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                serde_json::Value: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<serde_json::Value>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
            {
                use sqlxplus::Model;
                use sqlxplus::DatabaseInfo;
                let table = Self::TABLE;
                let pk = Self::PK;
                let escaped_table = DB::escape_identifier(table);
                let escaped_pk = DB::escape_identifier(pk);

                // 构建 UPDATE SET 子句（Patch 语义）
                let mut set_parts: Vec<String> = Vec::new();
                let mut placeholder_index = 0;

                // 非 Option 字段
                #(
                    set_parts.push(format!("{} = {}", #update_normal_field_columns, DB::placeholder(placeholder_index)));
                    placeholder_index += 1;
                )*

                // Option 字段
                #(
                    if self.#update_option_field_names.is_some() {
                        set_parts.push(format!("{} = {}", #update_option_field_columns, DB::placeholder(placeholder_index)));
                        placeholder_index += 1;
                    }
                )*

                if set_parts.is_empty() {
                    return Ok(());
                }

                let sql = format!(
                    "UPDATE {} SET {} WHERE {} = {}",
                    escaped_table,
                    set_parts.join(", "),
                    escaped_pk,
                    DB::placeholder(placeholder_index)
                );

                let mut query = sqlx::query(&sql);
                // 非 Option 字段：始终绑定
                #(
                    query = query.bind(&self.#update_normal_field_names);
                )*
                // Option 字段：仅当为 Some 时绑定
                #(
                    if let Some(ref val) = self.#update_option_field_names {
                        query = query.bind(val);
                    }
                )*
                query = query.bind(&self.#pk_ident);
                query.execute(executor).await?;
                Ok(())
            }

            // 泛型版本的 update_with_none
            async fn update_with_none<'e, 'c: 'e, DB, E>(&self, executor: E) -> sqlxplus::Result<()>
            where
                DB: sqlx::Database + sqlxplus::DatabaseInfo,
                for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
                E: sqlx::Executor<'c, Database = DB> + Send,
                // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
                String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<i64>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<i32>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<i16>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                chrono::DateTime<chrono::Utc>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<chrono::DateTime<chrono::Utc>>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                serde_json::Value: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
                Option<serde_json::Value>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
            {
                use sqlxplus::Model;
                use sqlxplus::DatabaseInfo;
                use sqlxplus::db_pool::DbDriver;
                let table = Self::TABLE;
                let pk = Self::PK;
                let escaped_table = DB::escape_identifier(table);
                let escaped_pk = DB::escape_identifier(pk);

                // 构建 UPDATE SET 子句（Reset 语义）
                let mut set_parts: Vec<String> = Vec::new();
                let mut placeholder_index = 0;

                // 非 Option 字段：始终更新为当前值
                #(
                    set_parts.push(format!("{} = {}", #update_normal_field_columns, DB::placeholder(placeholder_index)));
                    placeholder_index += 1;
                )*

                // Option 字段：根据数据库类型处理
                match DB::get_driver() {
                    DbDriver::Sqlite => {
                        // SQLite 不支持 DEFAULT，跳过 None 字段
                        #(
                            if self.#update_option_field_names.is_some() {
                                set_parts.push(format!("{} = {}", #update_option_field_columns, DB::placeholder(placeholder_index)));
                                placeholder_index += 1;
                            }
                        )*
                    }
                    _ => {
                        // MySQL 和 PostgreSQL 使用 DEFAULT
                        #(
                            if self.#update_option_field_names.is_some() {
                                set_parts.push(format!("{} = {}", #update_option_field_columns, DB::placeholder(placeholder_index)));
                                placeholder_index += 1;
                            } else {
                                set_parts.push(format!("{} = DEFAULT", #update_option_field_columns));
                            }
                        )*
                    }
                }

                if set_parts.is_empty() {
                    return Ok(());
                }

                let sql = format!(
                    "UPDATE {} SET {} WHERE {} = {}",
                    escaped_table,
                    set_parts.join(", "),
                    escaped_pk,
                    DB::placeholder(placeholder_index)
                );

                let mut query = sqlx::query(&sql);
                // 非 Option 字段：始终绑定
                #(
                    query = query.bind(&self.#update_normal_field_names);
                )*
                // Option 字段：仅当为 Some 时绑定（None 使用 DEFAULT 或跳过）
                #(
                    if let Some(ref val) = self.#update_option_field_names {
                        query = query.bind(val);
                    }
                )*
                query = query.bind(&self.#pk_ident);
                query.execute(executor).await?;
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
