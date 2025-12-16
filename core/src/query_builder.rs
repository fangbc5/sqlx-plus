use crate::db_pool::DbDriver;
use crate::utils::escape_identifier;
use std::fmt::Write;

/// 绑定值，用于安全地传递参数
#[derive(Debug, Clone, PartialEq)]
pub enum BindValue {
    String(String),
    Int64(i64),
    Int32(i32),
    Int16(i16),
    Float64(f64),
    Float32(f32),
    Bool(bool),
    Null,
}

impl BindValue {
    pub fn to_sql_value(&self) -> String {
        match self {
            BindValue::String(s) => format!("'{}'", s.replace("'", "''")),
            BindValue::Int64(i) => i.to_string(),
            BindValue::Int32(i) => i.to_string(),
            BindValue::Int16(i) => i.to_string(),
            BindValue::Float64(f) => f.to_string(),
            BindValue::Float32(f) => f.to_string(),
            BindValue::Bool(b) => b.to_string(),
            BindValue::Null => "NULL".to_string(),
        }
    }
}

/// 条件类型：AND 或 OR
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConditionType {
    And,
    Or,
}

/// 条件项：可以是单个条件或条件组
#[derive(Debug, Clone)]
enum ConditionItem {
    /// 单个条件：(field, operator, condition_type)
    /// 注意：绑定值存储在 QueryBuilder 的 binds 字段中，不在此处
    Single(String, String, ConditionType),
    /// 条件组：(嵌套的 QueryBuilder, condition_type)
    Group(Box<QueryBuilder>, ConditionType),
}

/// 安全的查询构建器，使用绑定参数而非字符串拼接
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    base_sql: String,
    conditions: Vec<ConditionItem>,
    order_by: Vec<(String, bool)>, // (field, ascending)
    binds: Vec<BindValue>,
    // 仅用于 into_sql 链式场景；在 into_paginated_sql 中会被显式忽略
    limit: Option<u64>,
    offset: Option<u64>,
    // GROUP BY 和 HAVING 支持
    group_by: Vec<String>,
    having_conditions: Vec<ConditionItem>,
    having_binds: Vec<BindValue>,
}

impl QueryBuilder {
    pub fn new(base_sql: impl Into<String>) -> Self {
        Self {
            base_sql: base_sql.into(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            binds: Vec::new(),
            limit: None,
            offset: None,
            group_by: Vec::new(),
            having_conditions: Vec::new(),
            having_binds: Vec::new(),
        }
    }

    /// 替换基础 SQL（保留已有的条件、排序和绑定）
    pub fn with_base_sql(mut self, base_sql: impl Into<String>) -> Self {
        self.base_sql = base_sql.into();
        self
    }

    /// 设置 LIMIT（链式调用），仅作用于 into_sql 生成的 SQL
    pub fn limit(mut self, n: u64) -> Self {
        self.limit = Some(n);
        self
    }

    /// 设置 OFFSET（链式调用），仅作用于 into_sql 生成的 SQL
    pub fn offset(mut self, n: u64) -> Self {
        self.offset = Some(n);
        self
    }

    /// 添加 GROUP BY 字段（链式调用，可多次调用添加多个字段）
    pub fn group_by(mut self, field: &str) -> Self {
        self.group_by.push(field.to_string());
        self
    }

    /// HAVING 条件：等于
    pub fn having_eq(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.having_conditions.push(ConditionItem::Single(
            field.to_string(),
            "=".to_string(),
            ConditionType::And,
        ));
        self.having_binds.push(bind_value);
        self
    }

    /// HAVING 条件：不等于
    pub fn having_ne(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.having_conditions.push(ConditionItem::Single(
            field.to_string(),
            "!=".to_string(),
            ConditionType::And,
        ));
        self.having_binds.push(bind_value);
        self
    }

    /// HAVING 条件：大于
    pub fn having_gt(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.having_conditions.push(ConditionItem::Single(
            field.to_string(),
            ">".to_string(),
            ConditionType::And,
        ));
        self.having_binds.push(bind_value);
        self
    }

    /// HAVING 条件：大于等于
    pub fn having_ge(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.having_conditions.push(ConditionItem::Single(
            field.to_string(),
            ">=".to_string(),
            ConditionType::And,
        ));
        self.having_binds.push(bind_value);
        self
    }

    /// HAVING 条件：小于
    pub fn having_lt(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.having_conditions.push(ConditionItem::Single(
            field.to_string(),
            "<".to_string(),
            ConditionType::And,
        ));
        self.having_binds.push(bind_value);
        self
    }

    /// HAVING 条件：小于等于
    pub fn having_le(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.having_conditions.push(ConditionItem::Single(
            field.to_string(),
            "<=".to_string(),
            ConditionType::And,
        ));
        self.having_binds.push(bind_value);
        self
    }

    pub fn and_eq(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "=".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn and_ne(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "!=".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn and_gt(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            ">".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn and_ge(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            ">=".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn and_lt(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "<".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn and_le(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "<=".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    // ========== OR 条件方法 ==========
    pub fn or_eq(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "=".to_string(),
            ConditionType::Or,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn or_ne(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "!=".to_string(),
            ConditionType::Or,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn or_gt(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            ">".to_string(),
            ConditionType::Or,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn or_ge(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            ">=".to_string(),
            ConditionType::Or,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn or_lt(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "<".to_string(),
            ConditionType::Or,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn or_le(mut self, field: &str, value: impl Into<BindValue>) -> Self {
        let bind_value = value.into();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "<=".to_string(),
            ConditionType::Or,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn and_like(mut self, field: &str, value: impl Into<String>) -> Self {
        let s = value.into();
        let bind_value = BindValue::String(format!("%{}%", s));
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "LIKE".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    /// LIKE 前缀匹配（value%）
    pub fn and_like_prefix(mut self, field: &str, value: impl Into<String>) -> Self {
        let s = value.into();
        let bind_value = BindValue::String(format!("{}%", s));
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "LIKE".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    /// LIKE 后缀匹配（%value）
    pub fn and_like_suffix(mut self, field: &str, value: impl Into<String>) -> Self {
        let s = value.into();
        let bind_value = BindValue::String(format!("%{}", s));
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "LIKE".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    /// LIKE 精确匹配（不添加 %）
    pub fn and_like_exact(mut self, field: &str, value: impl Into<String>) -> Self {
        let s = value.into();
        let bind_value = BindValue::String(s);
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "LIKE".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    /// LIKE 自定义模式匹配
    pub fn and_like_custom(mut self, field: &str, pattern: impl Into<String>) -> Self {
        let s = pattern.into();
        let bind_value = BindValue::String(s);
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "LIKE".to_string(),
            ConditionType::And,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn or_like(mut self, field: &str, value: impl Into<String>) -> Self {
        let s = value.into();
        let bind_value = BindValue::String(format!("%{}%", s));
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "LIKE".to_string(),
            ConditionType::Or,
        ));
        self.binds.push(bind_value);
        self
    }

    pub fn and_in(mut self, field: &str, values: Vec<impl Into<BindValue>>) -> Self {
        let bind_values: Vec<BindValue> = values.into_iter().map(|v| v.into()).collect();
        let start_index = self.binds.len();
        for bv in &bind_values {
            self.binds.push(bv.clone());
        }
        let end_index = self.binds.len();
        // 存储 IN 子句的起始和结束索引
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            format!("IN({},{})", start_index, end_index),
            ConditionType::And,
        ));
        self
    }

    /// NOT IN 查询
    pub fn and_not_in(mut self, field: &str, values: Vec<impl Into<BindValue>>) -> Self {
        let bind_values: Vec<BindValue> = values.into_iter().map(|v| v.into()).collect();
        let start_index = self.binds.len();
        for bv in &bind_values {
            self.binds.push(bv.clone());
        }
        let end_index = self.binds.len();
        // 存储 NOT IN 子句的起始和结束索引
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            format!("NOT_IN({},{})", start_index, end_index),
            ConditionType::And,
        ));
        self
    }

    pub fn or_in(mut self, field: &str, values: Vec<impl Into<BindValue>>) -> Self {
        let bind_values: Vec<BindValue> = values.into_iter().map(|v| v.into()).collect();
        let start_index = self.binds.len();
        for bv in &bind_values {
            self.binds.push(bv.clone());
        }
        let end_index = self.binds.len();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            format!("IN({},{})", start_index, end_index),
            ConditionType::Or,
        ));
        self
    }

    /// IS NULL 查询
    pub fn and_is_null(mut self, field: &str) -> Self {
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "IS NULL".to_string(),
            ConditionType::And,
        ));
        self
    }

    /// IS NOT NULL 查询
    pub fn and_is_not_null(mut self, field: &str) -> Self {
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "IS NOT NULL".to_string(),
            ConditionType::And,
        ));
        self
    }

    pub fn or_is_null(mut self, field: &str) -> Self {
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "IS NULL".to_string(),
            ConditionType::Or,
        ));
        self
    }

    pub fn or_is_not_null(mut self, field: &str) -> Self {
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            "IS NOT NULL".to_string(),
            ConditionType::Or,
        ));
        self
    }

    /// BETWEEN 范围查询
    pub fn and_between(
        mut self,
        field: &str,
        min: impl Into<BindValue>,
        max: impl Into<BindValue>,
    ) -> Self {
        let min_value = min.into();
        let max_value = max.into();
        let start_index = self.binds.len();
        self.binds.push(min_value.clone());
        self.binds.push(max_value.clone());
        let end_index = self.binds.len();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            format!("BETWEEN({},{})", start_index, end_index),
            ConditionType::And,
        ));
        self
    }

    pub fn or_between(
        mut self,
        field: &str,
        min: impl Into<BindValue>,
        max: impl Into<BindValue>,
    ) -> Self {
        let min_value = min.into();
        let max_value = max.into();
        let start_index = self.binds.len();
        self.binds.push(min_value.clone());
        self.binds.push(max_value.clone());
        let end_index = self.binds.len();
        self.conditions.push(ConditionItem::Single(
            field.to_string(),
            format!("BETWEEN({},{})", start_index, end_index),
            ConditionType::Or,
        ));
        self
    }

    /// AND 条件分组：创建一个用 AND 连接的条件组
    /// 示例：`builder.and_group(|b| b.and_eq("a", 1).and_eq("b", 2))`
    /// 生成：`(a = ? AND b = ?)`
    pub fn and_group<F>(mut self, f: F) -> Self
    where
        F: FnOnce(QueryBuilder) -> QueryBuilder,
    {
        let group_builder = f(QueryBuilder::new(""));
        // 合并嵌套 builder 的 binds
        for bind in &group_builder.binds {
            self.binds.push(bind.clone());
        }
        self.conditions.push(ConditionItem::Group(
            Box::new(group_builder),
            ConditionType::And,
        ));
        self
    }

    /// OR 条件分组：创建一个用 OR 连接的条件组
    /// 示例：`builder.or_group(|b| b.and_eq("c", 3).and_eq("d", 4))`
    /// 生成：`(c = ? OR d = ?)`
    pub fn or_group<F>(mut self, f: F) -> Self
    where
        F: FnOnce(QueryBuilder) -> QueryBuilder,
    {
        let group_builder = f(QueryBuilder::new(""));
        // 合并嵌套 builder 的 binds
        for bind in &group_builder.binds {
            self.binds.push(bind.clone());
        }
        self.conditions.push(ConditionItem::Group(
            Box::new(group_builder),
            ConditionType::Or,
        ));
        self
    }

    pub fn order_by(mut self, field: &str, ascending: bool) -> Self {
        self.order_by.push((field.to_string(), ascending));
        self
    }

    /// 生成条件部分的 SQL（不包含 base_sql 和 ORDER BY）
    /// 返回 (sql, bind_count)
    fn build_conditions_sql(&self, driver: DbDriver, start_bind_index: usize) -> (String, usize) {
        if self.conditions.is_empty() {
            return (String::new(), start_bind_index);
        }

        let mut sql = String::new();
        let mut bind_index = start_bind_index;
        let mut first = true;
        let mut prev_condition_type = ConditionType::And;

        for item in &self.conditions {
            let condition_type = match item {
                ConditionItem::Single(_, _, ct) => *ct,
                ConditionItem::Group(_, ct) => *ct,
            };

            // 处理条件连接符（AND 或 OR）
            if !first {
                if prev_condition_type == ConditionType::And && condition_type == ConditionType::Or
                {
                    sql.push_str(" OR ");
                } else if prev_condition_type == ConditionType::Or
                    && condition_type == ConditionType::And
                {
                    sql.push_str(" AND ");
                } else if condition_type == ConditionType::Or {
                    sql.push_str(" OR ");
                } else {
                    sql.push_str(" AND ");
                }
            }
            first = false;
            prev_condition_type = condition_type;

            match item {
                ConditionItem::Single(field, op, _) => {
                    // 对列名进行转义，兼容 MySQL / Postgres / SQLite
                    let escaped_field = escape_identifier(driver, field);

                    // 处理特殊操作符
                    if op.starts_with("IN(") {
                        let parts: Vec<&str> = op
                            .strip_prefix("IN(")
                            .unwrap()
                            .strip_suffix(")")
                            .unwrap()
                            .split(',')
                            .collect();
                        if parts.len() == 2 {
                            let start: usize = parts[0].parse().unwrap_or(0);
                            let end: usize = parts[1].parse().unwrap_or(start);
                            sql.push_str(&format!("{} IN (", escaped_field));
                            let mut in_first = true;
                            for _ in start..end {
                                if !in_first {
                                    sql.push_str(", ");
                                }
                                in_first = false;
                                sql.push_str(&driver.placeholder(bind_index));
                                bind_index += 1;
                            }
                            sql.push_str(")");
                        }
                    } else if op.starts_with("NOT_IN(") {
                        let parts: Vec<&str> = op
                            .strip_prefix("NOT_IN(")
                            .unwrap()
                            .strip_suffix(")")
                            .unwrap()
                            .split(',')
                            .collect();
                        if parts.len() == 2 {
                            let start: usize = parts[0].parse().unwrap_or(0);
                            let end: usize = parts[1].parse().unwrap_or(start);
                            sql.push_str(&format!("{} NOT IN (", escaped_field));
                            let mut in_first = true;
                            for _ in start..end {
                                if !in_first {
                                    sql.push_str(", ");
                                }
                                in_first = false;
                                sql.push_str(&driver.placeholder(bind_index));
                                bind_index += 1;
                            }
                            sql.push_str(")");
                        }
                    } else if op.starts_with("BETWEEN(") {
                        let parts: Vec<&str> = op
                            .strip_prefix("BETWEEN(")
                            .unwrap()
                            .strip_suffix(")")
                            .unwrap()
                            .split(',')
                            .collect();
                        if parts.len() == 2 {
                            sql.push_str(&format!(
                                "{} BETWEEN {} AND {}",
                                escaped_field,
                                driver.placeholder(bind_index),
                                driver.placeholder(bind_index + 1)
                            ));
                            bind_index += 2;
                        }
                    } else if op == "IS NULL" || op == "IS NOT NULL" {
                        sql.push_str(&format!("{} {}", escaped_field, op));
                    } else {
                        sql.push_str(&format!(
                            "{} {} {}",
                            escaped_field,
                            op,
                            driver.placeholder(bind_index)
                        ));
                        bind_index += 1;
                    }
                }
                ConditionItem::Group(group_builder, _) => {
                    // 递归处理分组条件
                    sql.push_str("(");
                    let (group_sql, new_bind_index) =
                        group_builder.build_conditions_sql(driver, bind_index);
                    sql.push_str(&group_sql);
                    sql.push_str(")");
                    bind_index = new_bind_index;
                }
            }
        }

        (sql, bind_index)
    }

    /// 生成 HAVING 条件部分的 SQL（不包含 base_sql、WHERE、GROUP BY 和 ORDER BY）
    /// 返回 (sql, bind_count)
    fn build_having_sql(&self, driver: DbDriver, start_bind_index: usize) -> (String, usize) {
        if self.having_conditions.is_empty() {
            return (String::new(), start_bind_index);
        }

        // 创建一个临时的 QueryBuilder 来复用 build_conditions_sql 的逻辑
        // 但使用 having_conditions 和 having_binds
        let mut temp_builder = QueryBuilder::new("");
        temp_builder.conditions = self.having_conditions.clone();
        temp_builder.binds = self.having_binds.clone();

        temp_builder.build_conditions_sql(driver, start_bind_index)
    }

    pub fn into_sql(&self, driver: DbDriver) -> String {
        let mut sql = self.base_sql.clone();

        // 添加 WHERE 条件
        if !self.conditions.is_empty() {
            // 检查 base_sql 是否已经包含 WHERE
            let base_upper = sql.to_uppercase();
            let has_where = base_upper.contains(" WHERE ");

            if !has_where {
                sql.push_str(" WHERE ");
            } else {
                sql.push_str(" AND ");
            }

            let (conditions_sql, _) = self.build_conditions_sql(driver, 0);
            sql.push_str(&conditions_sql);
        }

        // 添加 GROUP BY
        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            for (i, field) in self.group_by.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                let escaped_field = escape_identifier(driver, field);
                sql.push_str(&escaped_field);
            }
        }

        // 添加 HAVING 条件
        if !self.having_conditions.is_empty() {
            sql.push_str(" HAVING ");
            // 构建 HAVING 条件 SQL，需要从 WHERE 条件的绑定索引之后开始
            let where_bind_count = self.binds.len();
            let (having_sql, _) = self.build_having_sql(driver, where_bind_count);
            sql.push_str(&having_sql);
        }

        // 添加 ORDER BY
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            for (i, (field, ascending)) in self.order_by.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ");
                }
                let escaped_field = escape_identifier(driver, field);
                sql.push_str(&escaped_field);
                if !ascending {
                    sql.push_str(" DESC");
                }
            }
        }

        // 如果设置了 limit / offset，则追加
        if let Some(limit) = self.limit {
            write!(sql, " LIMIT {}", limit).unwrap();
            if let Some(offset) = self.offset {
                write!(sql, " OFFSET {}", offset).unwrap();
            }
        }

        driver.convert_placeholders(&sql)
    }

    pub fn into_count_sql(&self, driver: DbDriver) -> String {
        // 将 SELECT ... FROM 转换为 SELECT COUNT(*) FROM
        let base = self.base_sql.clone();
        let count_sql = if let Some(from_pos) = base.to_uppercase().find(" FROM ") {
            format!("SELECT COUNT(*){}", &base[from_pos..])
        } else {
            format!("SELECT COUNT(*) FROM ({}) AS count_query", base)
        };

        let mut builder = QueryBuilder::new(count_sql);
        builder.conditions = self.conditions.clone();
        builder.binds = self.binds.clone();
        builder.into_sql(driver)
    }

    pub fn into_paginated_sql(&self, driver: DbDriver, limit: u64, offset: u64) -> String {
        // 分页时，page/size（limit/offset 参数）应当具有最高优先级，
        // 因此忽略构建器上通过链式设置的 limit / offset，避免重复附加。
        let mut builder = self.clone();
        builder.limit = None;
        builder.offset = None;
        let mut sql = builder.into_sql(driver);

        match driver {
            DbDriver::MySql | DbDriver::Sqlite => {
                write!(sql, " LIMIT {} OFFSET {}", limit, offset).unwrap();
            }
            DbDriver::Postgres => {
                write!(sql, " LIMIT {} OFFSET {}", limit, offset).unwrap();
            }
        }
        sql
    }

    /// 返回所有绑定值（WHERE 条件 + HAVING 条件）
    pub fn binds(&self) -> Vec<BindValue> {
        let mut all_binds = self.binds.clone();
        all_binds.extend_from_slice(&self.having_binds);
        all_binds
    }
}

impl From<String> for BindValue {
    fn from(s: String) -> Self {
        BindValue::String(s)
    }
}

impl From<&str> for BindValue {
    fn from(s: &str) -> Self {
        BindValue::String(s.to_string())
    }
}

impl From<i64> for BindValue {
    fn from(i: i64) -> Self {
        BindValue::Int64(i)
    }
}

impl From<i32> for BindValue {
    fn from(i: i32) -> Self {
        BindValue::Int32(i)
    }
}

impl From<i16> for BindValue {
    fn from(i: i16) -> Self {
        BindValue::Int16(i)
    }
}

impl From<f64> for BindValue {
    fn from(f: f64) -> Self {
        BindValue::Float64(f)
    }
}

impl From<f32> for BindValue {
    fn from(f: f32) -> Self {
        BindValue::Float32(f)
    }
}

impl From<bool> for BindValue {
    fn from(b: bool) -> Self {
        BindValue::Bool(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalize(sql: &str) -> String {
        sql.replace('`', "").replace('\"', "")
    }

    fn mysql_driver() -> DbDriver {
        DbDriver::MySql
    }

    fn postgres_driver() -> DbDriver {
        DbDriver::Postgres
    }

    fn sqlite_driver() -> DbDriver {
        DbDriver::Sqlite
    }

    // ========== 基本条件查询测试 ==========
    #[test]
    fn test_and_eq() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("id", 1)
            .and_eq("name", "test");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE id = ? AND name = ?"
        );
        assert_eq!(builder.binds().len(), 2);
    }

    #[test]
    fn test_and_ne() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_ne("status", 0);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(normalize(&sql), "SELECT * FROM users WHERE status != ?");
    }

    #[test]
    fn test_and_gt_ge_lt_le() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_gt("age", 18)
            .and_ge("score", 60)
            .and_lt("age", 65)
            .and_le("score", 100);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE age > ? AND score >= ? AND age < ? AND score <= ?"
        );
    }

    // ========== OR 条件查询测试 ==========
    #[test]
    fn test_or_conditions() {
        let builder = QueryBuilder::new("SELECT * FROM user")
            .and_eq("is_del", 0)
            .or_eq("id", 1)
            .or_eq("id", 2);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM user WHERE is_del = ? OR id = ? OR id = ?"
        );
        assert_eq!(builder.binds().len(), 3);
    }

    #[test]
    fn test_or_gt_lt() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .or_gt("age", 18)
            .or_lt("age", 65);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE age > ? OR age < ?"
        );
    }

    // ========== LIKE 查询测试 ==========
    #[test]
    fn test_and_like() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_like("name", "test");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(normalize(&sql), "SELECT * FROM users WHERE name LIKE ?");
    }

    #[test]
    fn test_and_like_prefix() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_like_prefix("name", "test");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(normalize(&sql), "SELECT * FROM users WHERE name LIKE ?");
        assert_eq!(builder.binds()[0], BindValue::String("test%".to_string()));
    }

    #[test]
    fn test_and_like_suffix() {
        let builder =
            QueryBuilder::new("SELECT * FROM users").and_like_suffix("email", "@example.com");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(normalize(&sql), "SELECT * FROM users WHERE email LIKE ?");
        assert_eq!(
            builder.binds()[0],
            BindValue::String("%@example.com".to_string())
        );
    }

    #[test]
    fn test_and_like_exact() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_like_exact("name", "admin");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(normalize(&sql), "SELECT * FROM users WHERE name LIKE ?");
        assert_eq!(builder.binds()[0], BindValue::String("admin".to_string()));
    }

    #[test]
    fn test_and_like_custom() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_like_custom("name", "test_%");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(normalize(&sql), "SELECT * FROM users WHERE name LIKE ?");
        assert_eq!(builder.binds()[0], BindValue::String("test_%".to_string()));
    }

    #[test]
    fn test_or_like() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1")
            .and_eq("is_del", 0)
            .or_like("name", "test");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE 1=1 AND is_del = ? OR name LIKE ?"
        );
    }

    // ========== IN / NOT IN 查询测试 ==========
    #[test]
    fn test_and_in() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_in("id", vec![1, 2, 3]);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(normalize(&sql), "SELECT * FROM users WHERE id IN (?, ?, ?)");
        assert_eq!(builder.binds().len(), 3);
    }

    #[test]
    fn test_and_not_in() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_not_in("id", vec![1, 2, 3]);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE id NOT IN (?, ?, ?)"
        );
        assert_eq!(builder.binds().len(), 3);
    }

    #[test]
    fn test_or_in() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1")
            .and_eq("is_del", 0)
            .or_in("status", vec![1, 2]);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE 1=1 AND is_del = ? OR status IN (?, ?)"
        );
    }

    // ========== IS NULL / IS NOT NULL 测试 ==========
    #[test]
    fn test_and_is_null() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_is_null("deleted_at");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE deleted_at IS NULL"
        );
    }

    #[test]
    fn test_and_is_not_null() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_is_not_null("updated_at");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE updated_at IS NOT NULL"
        );
    }

    #[test]
    fn test_or_is_null() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1")
            .and_eq("is_del", 0)
            .or_is_null("deleted_at");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE 1=1 AND is_del = ? OR deleted_at IS NULL"
        );
    }

    #[test]
    fn test_or_is_not_null() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1")
            .and_eq("is_del", 0)
            .or_is_not_null("updated_at");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE 1=1 AND is_del = ? OR updated_at IS NOT NULL"
        );
    }

    // ========== BETWEEN 查询测试 ==========
    #[test]
    fn test_and_between() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_between("age", 18, 65);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE age BETWEEN ? AND ?"
        );
        assert_eq!(builder.binds().len(), 2);
    }

    #[test]
    fn test_or_between() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1")
            .and_eq("is_del", 0)
            .or_between("score", 60, 100);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE 1=1 AND is_del = ? OR score BETWEEN ? AND ?"
        );
    }

    // ========== 条件分组测试 ==========
    #[test]
    fn test_and_group() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_group(|b| b.and_eq("is_del", 0).and_eq("status", 1));
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE (is_del = ? AND status = ?)"
        );
        assert_eq!(builder.binds().len(), 2);
    }

    #[test]
    fn test_or_group() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1")
            .and_eq("id", 1)
            .or_group(|b| b.and_eq("status", "active").and_eq("is_del", 0));
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE 1=1 AND id = ? OR (status = ? AND is_del = ?)"
        );
    }

    #[test]
    fn test_complex_grouping() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1")
            .and_group(|b| b.or_eq("id", 1).or_eq("id", 2))
            .and_group(|b| b.or_eq("id", 3).or_eq("id", 4));
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE 1=1 AND (id = ? OR id = ?) AND (id = ? OR id = ?)"
        );
        assert_eq!(builder.binds().len(), 4);
    }

    #[test]
    fn test_mixed_conditions_and_groups() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1")
            .and_eq("is_del", 0)
            .and_group(|b| b.and_eq("id", 1).or_eq("id", 2))
            .or_group(|b| b.and_eq("id", 3).and_eq("is_del", 0));
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE 1=1 AND is_del = ? AND (id = ? OR id = ?) OR (id = ? AND is_del = ?)"
        );
        assert_eq!(builder.binds().len(), 5);
    }

    #[test]
    fn test_nested_groups() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1").and_group(|b| {
            b.and_eq("status", 1)
                .and_group(|b2| b2.or_eq("type", "admin").or_eq("type", "user"))
        });
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE 1=1 AND (status = ? AND (type = ? OR type = ?))"
        );
        assert_eq!(builder.binds().len(), 3);
    }

    // ========== ORDER BY 测试 ==========
    #[test]
    fn test_order_by() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("is_del", 0)
            .order_by("id", true)
            .order_by("name", false);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE is_del = ? ORDER BY id, name DESC"
        );
    }

    #[test]
    fn test_order_by_multiple() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .order_by("created_at", false)
            .order_by("id", true);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users ORDER BY created_at DESC, id"
        );
    }

    // ========== COUNT SQL 测试 ==========
    #[test]
    fn test_into_count_sql() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("is_del", 0)
            .and_eq("status", 1);
        let count_sql = builder.into_count_sql(mysql_driver());
        assert_eq!(
            normalize(&count_sql),
            "SELECT COUNT(*) FROM users WHERE is_del = ? AND status = ?"
        );
    }

    #[test]
    fn test_into_count_sql_with_where() {
        let builder = QueryBuilder::new("SELECT id, name FROM users WHERE 1=1").and_eq("is_del", 0);
        let count_sql = builder.into_count_sql(mysql_driver());
        assert_eq!(
            normalize(&count_sql),
            "SELECT COUNT(*) FROM users WHERE 1=1 AND is_del = ?"
        );
    }

    // ========== 分页 SQL 测试 ==========
    #[test]
    fn test_into_paginated_sql() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("is_del", 0)
            .order_by("id", false);
        let paginated_sql = builder.into_paginated_sql(mysql_driver(), 10, 20);
        assert_eq!(
            normalize(&paginated_sql),
            "SELECT * FROM users WHERE is_del = ? ORDER BY id DESC LIMIT 10 OFFSET 20"
        );
    }

    #[test]
    fn test_into_paginated_sql_postgres() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_eq("is_del", 0);
        let paginated_sql = builder.into_paginated_sql(postgres_driver(), 5, 10);
        assert_eq!(
            normalize(&paginated_sql),
            "SELECT * FROM users WHERE is_del = $1 LIMIT 5 OFFSET 10"
        );
    }

    #[test]
    fn test_into_paginated_sql_sqlite() {
        let builder = QueryBuilder::new("SELECT * FROM users").and_eq("is_del", 0);
        let paginated_sql = builder.into_paginated_sql(sqlite_driver(), 20, 0);
        assert_eq!(
            normalize(&paginated_sql),
            "SELECT * FROM users WHERE is_del = ? LIMIT 20 OFFSET 0"
        );
    }

    #[test]
    fn test_into_paginated_sql_ignores_builder_limit_offset() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("is_del", 0)
            .limit(100)
            .offset(200);
        let paginated_sql = builder.into_paginated_sql(mysql_driver(), 10, 20);
        // paginate 的 limit/offset 优先，忽略 builder 上的 100/200
        assert_eq!(
            normalize(&paginated_sql),
            "SELECT * FROM users WHERE is_del = ? LIMIT 10 OFFSET 20"
        );
    }

    // ========== LIMIT / OFFSET 链式方法测试 ==========
    #[test]
    fn test_limit_and_offset_mysql() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("is_del", 0)
            .limit(10)
            .offset(20);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            sql,
            "SELECT * FROM users WHERE `is_del` = ? LIMIT 10 OFFSET 20"
        );
    }

    #[test]
    fn test_limit_postgres_with_placeholders() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("id", 1)
            .limit(5);
        let sql = builder.into_sql(postgres_driver());
        assert_eq!(sql, "SELECT * FROM users WHERE \"id\" = $1 LIMIT 5");
    }

    // ========== 参数绑定测试 ==========
    #[test]
    fn test_binds() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("id", 1i64)
            .and_eq("name", "test")
            .and_eq("age", 18i32);
        let binds = builder.binds();
        assert_eq!(binds.len(), 3);
        assert_eq!(binds[0], BindValue::Int64(1));
        assert_eq!(binds[1], BindValue::String("test".to_string()));
        assert_eq!(binds[2], BindValue::Int32(18));
    }

    #[test]
    fn test_binds_with_groups() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("id", 1i64)
            .and_group(|b| b.and_eq("status", 2i64).and_eq("type", 3i64));
        let binds = builder.binds();
        assert_eq!(binds.len(), 3);
        assert_eq!(binds[0], BindValue::Int64(1));
        assert_eq!(binds[1], BindValue::Int64(2));
        assert_eq!(binds[2], BindValue::Int64(3));
    }

    // ========== 复杂查询组合测试 ==========
    #[test]
    fn test_complex_query() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1")
            .and_eq("is_del", 0)
            .and_group(|b| b.and_eq("status", "active").or_eq("status", "pending"))
            .and_between("age", 18, 65)
            .and_not_in("id", vec![100, 200, 300])
            .and_like("name", "test")
            .order_by("created_at", false)
            .order_by("id", true);

        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE 1=1 AND is_del = ? AND (status = ? OR status = ?) AND age BETWEEN ? AND ? AND id NOT IN (?, ?, ?) AND name LIKE ? ORDER BY created_at DESC, id"
        );
        assert_eq!(builder.binds().len(), 9);
    }

    #[test]
    fn test_empty_conditions() {
        let builder = QueryBuilder::new("SELECT * FROM users");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(sql, "SELECT * FROM users");
        assert_eq!(builder.binds().len(), 0);
    }

    #[test]
    fn test_existing_where_clause() {
        let builder = QueryBuilder::new("SELECT * FROM users WHERE id > 0").and_eq("is_del", 0);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE id > 0 AND is_del = ?"
        );
    }

    // ========== 不同数据库驱动测试 ==========
    #[test]
    fn test_mysql_placeholders() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("id", 1)
            .and_eq("name", "test");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE id = ? AND name = ?"
        );
    }

    #[test]
    fn test_postgres_placeholders() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("id", 1)
            .and_eq("name", "test");
        let sql = builder.into_sql(postgres_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE id = $1 AND name = $2"
        );
    }

    #[test]
    fn test_sqlite_placeholders() {
        let builder = QueryBuilder::new("SELECT * FROM users")
            .and_eq("id", 1)
            .and_eq("name", "test");
        let sql = builder.into_sql(sqlite_driver());
        assert_eq!(
            normalize(&sql),
            "SELECT * FROM users WHERE id = ? AND name = ?"
        );
    }

    // ========== BindValue 转换测试 ==========
    #[test]
    fn test_bind_value_from_string() {
        let bv: BindValue = "test".to_string().into();
        assert!(matches!(bv, BindValue::String(_)));
    }

    #[test]
    fn test_bind_value_from_str() {
        let bv: BindValue = "test".into();
        assert!(matches!(bv, BindValue::String(_)));
    }

    #[test]
    fn test_bind_value_from_i64() {
        let bv: BindValue = 100i64.into();
        assert!(matches!(bv, BindValue::Int64(100)));
    }

    #[test]
    fn test_bind_value_from_i32() {
        let bv: BindValue = 50i32.into();
        assert!(matches!(bv, BindValue::Int32(50)));
    }

    #[test]
    fn test_bind_value_from_f64() {
        let bv: BindValue = 3.14f64.into();
        assert!(matches!(bv, BindValue::Float64(_)));
    }

    #[test]
    fn test_bind_value_from_bool() {
        let bv: BindValue = true.into();
        assert!(matches!(bv, BindValue::Bool(true)));
    }

    // ========== GROUP BY 和 HAVING 测试 ==========
    #[test]
    fn test_group_by_single_field() {
        let builder =
            QueryBuilder::new("SELECT category, COUNT(*) FROM products").group_by("category");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products GROUP BY `category`"
        );
    }

    #[test]
    fn test_group_by_multiple_fields() {
        let builder = QueryBuilder::new("SELECT category, status, COUNT(*) FROM products")
            .group_by("category")
            .group_by("status");
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            sql,
            "SELECT category, status, COUNT(*) FROM products GROUP BY `category`, `status`"
        );
    }

    #[test]
    fn test_group_by_with_postgres() {
        let builder =
            QueryBuilder::new("SELECT category, COUNT(*) FROM products").group_by("category");
        let sql = builder.into_sql(postgres_driver());
        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products GROUP BY \"category\""
        );
    }

    #[test]
    fn test_having_eq() {
        let builder = QueryBuilder::new("SELECT category, COUNT(*) FROM products")
            .group_by("category")
            .having_eq("COUNT(*)", 10i64);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products GROUP BY `category` HAVING `COUNT(*)` = ?"
        );
        assert_eq!(builder.binds().len(), 1);
        assert_eq!(builder.binds()[0], BindValue::Int64(10));
    }

    #[test]
    fn test_having_gt() {
        let builder = QueryBuilder::new("SELECT category, COUNT(*) FROM products")
            .group_by("category")
            .having_gt("COUNT(*)", 5i64);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products GROUP BY `category` HAVING `COUNT(*)` > ?"
        );
        assert_eq!(builder.binds().len(), 1);
        assert_eq!(builder.binds()[0], BindValue::Int64(5));
    }

    #[test]
    fn test_having_multiple_conditions() {
        let builder = QueryBuilder::new("SELECT category, COUNT(*) FROM products")
            .group_by("category")
            .having_gt("COUNT(*)", 5i64)
            .having_lt("COUNT(*)", 100i64);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products GROUP BY `category` HAVING `COUNT(*)` > ? AND `COUNT(*)` < ?"
        );
        assert_eq!(builder.binds().len(), 2);
        assert_eq!(builder.binds()[0], BindValue::Int64(5));
        assert_eq!(builder.binds()[1], BindValue::Int64(100));
    }

    #[test]
    fn test_group_by_with_where_and_having() {
        let builder = QueryBuilder::new("SELECT category, COUNT(*) FROM products")
            .and_eq("status", "active")
            .group_by("category")
            .having_ge("COUNT(*)", 10i64);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products WHERE `status` = ? GROUP BY `category` HAVING `COUNT(*)` >= ?"
        );
        assert_eq!(builder.binds().len(), 2);
        assert_eq!(builder.binds()[0], BindValue::String("active".to_string()));
        assert_eq!(builder.binds()[1], BindValue::Int64(10));
    }

    #[test]
    fn test_group_by_with_order_by() {
        let builder = QueryBuilder::new("SELECT category, COUNT(*) FROM products")
            .group_by("category")
            .having_gt("COUNT(*)", 5)
            .order_by("COUNT(*)", false);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products GROUP BY `category` HAVING `COUNT(*)` > ? ORDER BY `COUNT(*)` DESC"
        );
        assert_eq!(builder.binds().len(), 1);
    }

    #[test]
    fn test_group_by_with_limit() {
        let builder = QueryBuilder::new("SELECT category, COUNT(*) FROM products")
            .group_by("category")
            .having_gt("COUNT(*)", 5)
            .limit(10);
        let sql = builder.into_sql(mysql_driver());
        assert_eq!(
            sql,
            "SELECT category, COUNT(*) FROM products GROUP BY `category` HAVING `COUNT(*)` > ? LIMIT 10"
        );
        assert_eq!(builder.binds().len(), 1);
    }

    #[test]
    fn test_having_all_operators() {
        let builder = QueryBuilder::new("SELECT category, COUNT(*) FROM products")
            .group_by("category")
            .having_eq("COUNT(*)", 10)
            .having_ne("SUM(price)", 1000)
            .having_gt("AVG(price)", 50)
            .having_ge("MAX(price)", 200)
            .having_lt("MIN(price)", 10)
            .having_le("COUNT(*)", 100);
        let sql = builder.into_sql(mysql_driver());
        assert!(sql.contains("GROUP BY `category`"));
        assert!(sql.contains("HAVING"));
        assert_eq!(builder.binds().len(), 6);
    }

    #[test]
    fn test_group_by_postgres_escaping() {
        let builder = QueryBuilder::new("SELECT \"user\", COUNT(*) FROM orders").group_by("user");
        let sql = builder.into_sql(postgres_driver());
        assert_eq!(
            sql,
            "SELECT \"user\", COUNT(*) FROM orders GROUP BY \"user\""
        );
    }
}
