//! Update Builder - 支持指定字段和 WHERE 条件进行更新

use crate::database_info::DatabaseInfo;
use crate::error::{Result, SqlxPlusError};
use super::query_builder::{BindValue, QueryBuilder};
use crate::traits::Model;

/// Update Builder - 支持指定字段和 WHERE 条件
///
/// 注意：此 Builder 需要 Model 实现 `UpdateFields` trait（由 CRUD derive 宏自动生成）
pub struct UpdateBuilder<M: Model> {
    model: M,
    fields: Vec<String>,                 // 要更新的字段列表（空表示更新所有字段）
    where_builder: Option<QueryBuilder>, // WHERE 条件构建器
}

/// Trait 用于从 Model 中提取字段值
/// 此 trait 由 CRUD derive 宏自动实现
pub trait UpdateFields: Model {
    /// 根据字段名获取字段值并转换为 BindValue
    /// 如果字段不存在或值为 None（对于 Option 类型），返回 None
    fn get_field_value(&self, field_name: &str) -> Option<BindValue>;

    /// 获取所有非主键字段名
    fn get_all_field_names() -> &'static [&'static str];

    /// 检查字段是否存在
    fn has_field(field_name: &str) -> bool;
}

impl<M: Model> UpdateBuilder<M> {
    /// 创建 UpdateBuilder
    pub fn new(model: M) -> Self {
        Self {
            model,
            fields: Vec::new(),
            where_builder: None,
        }
    }

    /// 指定要更新的字段（可链式调用多次）
    pub fn field(mut self, field_name: &str) -> Self {
        self.fields.push(field_name.to_string());
        self
    }

    /// 指定多个要更新的字段
    pub fn fields(mut self, field_names: &[&str]) -> Self {
        self.fields
            .extend(field_names.iter().map(|s| s.to_string()));
        self
    }

    /// 添加 WHERE 条件（复用 QueryBuilder）
    pub fn condition<F>(mut self, f: F) -> Self
    where
        F: FnOnce(QueryBuilder) -> QueryBuilder,
    {
        let base_sql = format!("SELECT * FROM {}", M::TABLE);
        let builder = f(QueryBuilder::new(base_sql));
        self.where_builder = Some(builder);
        self
    }

    /// 执行更新
    pub async fn execute<'e, 'c: 'e, DB, E>(self, executor: E) -> Result<u64>
    where
        DB: sqlx::Database + DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: sqlx::Executor<'c, Database = DB> + Send,
        M: UpdateFields,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
    {
        let driver = DB::get_driver();
        let table = M::TABLE;
        let pk = M::PK;
        let escaped_table = DB::escape_identifier(table);
        let escaped_pk = DB::escape_identifier(pk);

        // 确定要更新的字段列表
        let fields_to_update = if self.fields.is_empty() {
            // 如果没有指定字段，更新所有非主键字段
            M::get_all_field_names()
                .iter()
                .filter(|&&name| name != pk)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        } else {
            // 验证字段是否存在
            for field_name in &self.fields {
                if !M::has_field(field_name) {
                    return Err(SqlxPlusError::InvalidField(format!(
                        "Field '{}' does not exist in model '{}'",
                        field_name, table
                    )));
                }
                if field_name == pk {
                    return Err(SqlxPlusError::InvalidField(format!(
                        "Cannot update primary key field '{}'",
                        pk
                    )));
                }
            }
            self.fields
        };

        if fields_to_update.is_empty() {
            return Ok(0);
        }

        // 构建 SET 子句
        let mut set_parts = Vec::new();
        let mut set_values = Vec::new();
        let mut placeholder_index = 0;

        for field_name in &fields_to_update {
            if let Some(bind_value) = self.model.get_field_value(field_name) {
                let escaped_field = DB::escape_identifier(field_name);
                set_parts.push(format!(
                    "{} = {}",
                    escaped_field,
                    DB::placeholder(placeholder_index)
                ));
                set_values.push(bind_value);
                placeholder_index += 1;
            }
            // 如果字段值为 None（对于 Option 类型），跳过该字段
        }

        if set_parts.is_empty() {
            return Ok(0);
        }

        // 构建 WHERE 子句
        let (where_clause, where_binds) = if let Some(where_builder) = &self.where_builder {
            let (where_sql, _) = where_builder.build_where_sql(driver, placeholder_index);
            let where_binds = where_builder.where_binds().to_vec();
            (where_sql, where_binds)
        } else {
            // 如果没有 WHERE 条件，使用主键条件
            // 需要从 model 中获取主键值
            if let Some(pk_value) = self.model.get_field_value(pk) {
                let where_sql = format!("{} = {}", escaped_pk, DB::placeholder(placeholder_index));
                (where_sql, vec![pk_value])
            } else {
                return Err(SqlxPlusError::InvalidField(format!(
                    "Primary key field '{}' is required for update but value is None",
                    pk
                )));
            }
        };

        // 构建完整的 SQL
        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            escaped_table,
            set_parts.join(", "),
            where_clause
        );

        // 执行更新
        let mut query = sqlx::query(&sql);

        // 绑定 SET 子句的值
        for bind_value in set_values {
            crate::apply_bind_value!(query, bind_value);
        }

        // 绑定 WHERE 子句的值
        for bind_value in where_binds {
            crate::apply_bind_value!(query, bind_value);
        }

        let result = query.execute(executor).await?;
        // 在泛型上下文中，使用 match 根据数据库类型获取 rows_affected
        let rows_affected = match DB::get_driver() {
            crate::db_pool::DbDriver::MySql => unsafe {
                use sqlx::mysql::MySqlQueryResult;
                let ptr: *const DB::QueryResult = &result;
                let mysql_ptr = ptr as *const MySqlQueryResult;
                (*mysql_ptr).rows_affected()
            },
            crate::db_pool::DbDriver::Postgres => unsafe {
                use sqlx::postgres::PgQueryResult;
                let ptr: *const DB::QueryResult = &result;
                let pg_ptr = ptr as *const PgQueryResult;
                (*pg_ptr).rows_affected()
            },
            crate::db_pool::DbDriver::Sqlite => unsafe {
                use sqlx::sqlite::SqliteQueryResult;
                let ptr: *const DB::QueryResult = &result;
                let sqlite_ptr = ptr as *const SqliteQueryResult;
                (*sqlite_ptr).rows_affected()
            },
        };
        Ok(rows_affected)
    }
}
