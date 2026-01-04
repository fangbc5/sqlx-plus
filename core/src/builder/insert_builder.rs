//! Insert Builder - 支持指定插入字段

use crate::database_info::DatabaseInfo;
use crate::error::{Result, SqlxPlusError};
use crate::traits::Model;

/// Insert Builder - 支持指定插入字段
///
/// 注意：此 Builder 需要 Model 实现 `UpdateFields` trait（由 CRUD derive 宏自动生成）
pub struct InsertBuilder<M: Model> {
    model: M,
    fields: Vec<String>,        // 要插入的字段列表（空表示插入所有非主键字段）
    ignore_fields: Vec<String>, // 忽略的字段（如主键、自动递增字段等）
}

use super::update_builder::UpdateFields;

impl<M: Model> InsertBuilder<M> {
    /// 创建 InsertBuilder
    pub fn new(model: M) -> Self {
        Self {
            model,
            fields: Vec::new(),
            ignore_fields: Vec::new(),
        }
    }

    /// 指定要插入的字段（可链式调用多次）
    pub fn field(mut self, field_name: &str) -> Self {
        self.fields.push(field_name.to_string());
        self
    }

    /// 指定多个要插入的字段
    pub fn fields(mut self, field_names: &[&str]) -> Self {
        self.fields
            .extend(field_names.iter().map(|s| s.to_string()));
        self
    }

    /// 忽略某些字段（如主键、自动递增字段）
    pub fn ignore_field(mut self, field_name: &str) -> Self {
        self.ignore_fields.push(field_name.to_string());
        self
    }

    /// 执行插入
    pub async fn execute<'e, 'c: 'e, DB, E>(self, executor: E) -> Result<crate::crud::Id>
    where
        DB: sqlx::Database + DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: sqlx::Executor<'c, Database = DB> + Send,
        M: UpdateFields,
        // 基本类型必须实现 Type<DB> 和 Encode<DB>（用于绑定值）
        String: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB> + for<'r> sqlx::Decode<'r, DB>,
        i32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        i16: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f64: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        f32: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        bool: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Vec<u8>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        Option<String>: sqlx::Type<DB> + for<'b> sqlx::Encode<'b, DB>,
        usize: sqlx::ColumnIndex<DB::Row>,
    {
        let table = M::TABLE;
        let pk = M::PK;
        let escaped_table = DB::escape_identifier(table);

        // 确定要插入的字段列表
        let fields_to_insert = if self.fields.is_empty() {
            // 如果没有指定字段，插入所有非主键字段
            M::get_all_field_names()
                .iter()
                .filter(|&&name| name != pk && !self.ignore_fields.contains(&name.to_string()))
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
                        "Cannot insert primary key field '{}'",
                        pk
                    )));
                }
                if self.ignore_fields.contains(field_name) {
                    return Err(SqlxPlusError::InvalidField(format!(
                        "Field '{}' is in ignore list",
                        field_name
                    )));
                }
            }
            self.fields
        };

        if fields_to_insert.is_empty() {
            return Err(SqlxPlusError::InvalidField(
                "No fields to insert".to_string(),
            ));
        }

        // 构建 INSERT 语句
        let mut field_names = Vec::new();
        let mut values = Vec::new();
        let mut placeholder_index = 0;

        for field_name in &fields_to_insert {
            if let Some(bind_value) = self.model.get_field_value(field_name) {
                let escaped_field = DB::escape_identifier(field_name);
                field_names.push(escaped_field);
                values.push((bind_value, placeholder_index));
                placeholder_index += 1;
            }
            // 如果字段值为 None（对于 Option 类型），跳过该字段
        }

        if field_names.is_empty() {
            return Err(SqlxPlusError::InvalidField(
                "No valid field values to insert".to_string(),
            ));
        }

        // 构建 SQL
        let fields_str = field_names.join(", ");
        let placeholders: Vec<String> = (0..values.len()).map(|i| DB::placeholder(i)).collect();
        let placeholders_str = placeholders.join(", ");

        // 根据数据库类型构建 SQL
        let (sql, use_returning) = match DB::get_driver() {
            crate::db_pool::DbDriver::Postgres => {
                // PostgreSQL 使用 RETURNING 子句
                let sql = format!(
                    "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
                    escaped_table, fields_str, placeholders_str, pk
                );
                (sql, true)
            }
            _ => {
                // MySQL 和 SQLite 不使用 RETURNING
                let sql = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    escaped_table, fields_str, placeholders_str
                );
                (sql, false)
            }
        };

        // 执行插入
        let mut query = sqlx::query(&sql);

        // 绑定值（按顺序）
        for (bind_value, _) in values {
            crate::apply_bind_value!(query, bind_value);
        }

        // 获取插入的 ID
        let id = if use_returning {
            // PostgreSQL: 使用 RETURNING 子句
            use sqlx::Row as _;
            let row = query.fetch_one(executor).await?;
            row.get::<i64, _>(0usize)
        } else {
            // MySQL 和 SQLite: 从 execute 结果中获取
            let result = query.execute(executor).await?;
            match DB::get_driver() {
                crate::db_pool::DbDriver::MySql => unsafe {
                    use sqlx::mysql::MySqlQueryResult;
                    let ptr: *const DB::QueryResult = &result;
                    let mysql_ptr = ptr as *const MySqlQueryResult;
                    (*mysql_ptr).last_insert_id() as i64
                },
                crate::db_pool::DbDriver::Sqlite => unsafe {
                    use sqlx::sqlite::SqliteQueryResult;
                    let ptr: *const DB::QueryResult = &result;
                    let sqlite_ptr = ptr as *const SqliteQueryResult;
                    (*sqlite_ptr).last_insert_rowid() as i64
                },
                _ => unreachable!(),
            }
        };

        Ok(id)
    }
}
