//! Delete Builder - 支持指定 WHERE 条件进行删除

use super::query_builder::QueryBuilder;
use crate::database_info::DatabaseInfo;
use crate::error::{Result, SqlxPlusError};
use crate::traits::Model;

/// Delete Builder - 支持指定 WHERE 条件
pub struct DeleteBuilder<M: Model> {
    _phantom: std::marker::PhantomData<M>,
    where_builder: Option<QueryBuilder>,
}

impl<M: Model> DeleteBuilder<M> {
    /// 创建 DeleteBuilder
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            where_builder: None,
        }
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

    /// 执行删除
    pub async fn execute<'e, 'c: 'e, DB, E>(self, executor: E) -> Result<u64>
    where
        DB: sqlx::Database + DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: sqlx::Executor<'c, Database = DB> + Send,
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
        let escaped_table = DB::escape_identifier(table);

        // 构建 WHERE 子句
        let (where_clause, where_binds) = if let Some(where_builder) = &self.where_builder {
            // 检查是否是 allow_delete_all 的情况（空的 QueryBuilder）
            if !where_builder.has_conditions() {
                // 允许删除所有记录
                (String::new(), Vec::new())
            } else {
                let (where_sql, _) = where_builder.build_where_sql(driver, 0);
                let where_binds = where_builder.where_binds().to_vec();
                (where_sql, where_binds)
            }
        } else {
            // 如果没有 WHERE 条件，返回错误（防止误删所有数据）
            return Err(SqlxPlusError::InvalidField(
                "Delete operation requires WHERE condition. Use allow_delete_all() to explicitly allow deleting all records.".to_string(),
            ));
        };

        // 构建 SQL
        let sql = if where_clause.is_empty() {
            format!("DELETE FROM {}", escaped_table)
        } else {
            format!("DELETE FROM {} WHERE {}", escaped_table, where_clause)
        };

        // 执行删除
        let mut query = sqlx::query(&sql);

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

    /// 允许删除所有记录（危险操作，需要明确调用）
    pub fn allow_delete_all(mut self) -> Self {
        // 创建一个空的 WHERE builder，表示允许删除所有
        // 在 execute 中会特殊处理
        self.where_builder = Some(QueryBuilder::new(""));
        self
    }
}
