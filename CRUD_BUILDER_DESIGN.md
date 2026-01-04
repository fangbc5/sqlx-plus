# CRUD Builder 通用设计方案

## 1. 设计目标

设计一个通用的 Builder 模式，支持：

- **Update Builder**: 指定要更新的字段和 WHERE 条件
- **Insert Builder**: 指定要插入的字段
- **Delete Builder**: 指定 WHERE 条件进行删除

## 2. 核心设计原则

### 2.1 复用现有 QueryBuilder

- 复用 `QueryBuilder` 的 WHERE 条件构建能力
- 复用 `BindValue` 类型系统
- 保持 API 一致性

### 2.2 类型安全

- 利用 Rust 的类型系统确保编译期安全
- 支持泛型，适配不同数据库类型

### 2.3 链式调用

- 所有 Builder 支持链式调用
- 提供流畅的 API 体验

## 3. 架构设计

### 3.1 核心 Trait

```rust
/// CRUD Builder 的基础 trait
pub trait CrudBuilder<M: Model> {
    /// 执行操作
    async fn execute<'e, 'c: 'e, DB, E>(self, executor: E) -> Result<Self::Output>
    where
        DB: sqlx::Database + DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: sqlx::Executor<'c, Database = DB> + Send;

    /// 输出类型
    type Output;
}
```

### 3.2 UpdateBuilder 设计

```rust
/// Update Builder - 支持指定字段和 WHERE 条件
pub struct UpdateBuilder<M: Model> {
    model: M,
    fields: Vec<String>,  // 要更新的字段列表（空表示更新所有字段）
    where_builder: Option<QueryBuilder>,  // WHERE 条件构建器
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
        self.fields.extend(field_names.iter().map(|s| s.to_string()));
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
    async fn execute<'e, 'c: 'e, DB, E>(self, executor: E) -> Result<u64>
    where
        DB: sqlx::Database + DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: sqlx::Executor<'c, Database = DB> + Send,
    {
        // 1. 构建 SET 子句（只包含指定的字段）
        // 2. 构建 WHERE 子句（如果有）
        // 3. 执行 UPDATE 语句
        // 4. 返回受影响的行数
    }
}
```

### 3.3 InsertBuilder 设计

```rust
/// Insert Builder - 支持指定插入字段
pub struct InsertBuilder<M: Model> {
    model: M,
    fields: Vec<String>,  // 要插入的字段列表（空表示插入所有非主键字段）
    ignore_fields: Vec<String>,  // 忽略的字段（如主键、自动递增字段等）
}

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
        self.fields.extend(field_names.iter().map(|s| s.to_string()));
        self
    }

    /// 忽略某些字段（如主键、自动递增字段）
    pub fn ignore_field(mut self, field_name: &str) -> Self {
        self.ignore_fields.push(field_name.to_string());
        self
    }

    /// 执行插入
    async fn execute<'e, 'c: 'e, DB, E>(self, executor: E) -> Result<Id>
    where
        DB: sqlx::Database + DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: sqlx::Executor<'c, Database = DB> + Send,
    {
        // 1. 确定要插入的字段列表
        // 2. 构建 INSERT INTO ... (fields) VALUES (...) 语句
        // 3. 执行插入
        // 4. 返回插入的 ID
    }
}
```

### 3.4 DeleteBuilder 设计

```rust
/// Delete Builder - 支持指定 WHERE 条件
pub struct DeleteBuilder<M: Model> {
    where_builder: Option<QueryBuilder>,
}

impl<M: Model> DeleteBuilder<M> {
    /// 创建 DeleteBuilder
    pub fn new() -> Self {
        Self {
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
    async fn execute<'e, 'c: 'e, DB, E>(self, executor: E) -> Result<u64>
    where
        DB: sqlx::Database + DatabaseInfo,
        for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
        E: sqlx::Executor<'c, Database = DB> + Send,
    {
        // 1. 构建 DELETE FROM ... WHERE ... 语句
        // 2. 执行删除
        // 3. 返回受影响的行数
    }
}
```

## 4. API 使用示例

### 4.1 UpdateBuilder 使用示例

```rust
// 示例 1: 只更新指定字段，使用 WHERE 条件
let mut user = User { id: Some(1), username: "new_name".to_string(), email: "new@example.com".to_string(), ..Default::default() };
let affected = UpdateBuilder::new(user)
    .field("username")
    .field("email")
    .condition(|b| b.and_eq("id", 1).and_eq("status", "active"))
    .execute(pool)
    .await?;

// 示例 2: 更新所有字段，使用 WHERE 条件
let mut user = User { id: Some(1), username: "new_name".to_string(), ..Default::default() };
let affected = UpdateBuilder::new(user)
    .condition(|b| b.and_eq("id", 1))
    .execute(pool)
    .await?;

// 示例 3: 只更新指定字段，无 WHERE 条件（使用主键）
let mut user = User { id: Some(1), username: "new_name".to_string(), ..Default::default() };
let affected = UpdateBuilder::new(user)
    .field("username")
    .execute(pool)
    .await?;  // 如果没有 WHERE，默认使用主键条件
```

### 4.2 InsertBuilder 使用示例

```rust
// 示例 1: 插入所有字段（除了主键和自动递增字段）
let user = User { username: "test".to_string(), email: "test@example.com".to_string(), ..Default::default() };
let id = InsertBuilder::new(user)
    .execute(pool)
    .await?;

// 示例 2: 只插入指定字段
let user = User { username: "test".to_string(), email: "test@example.com".to_string(), age: Some(25), ..Default::default() };
let id = InsertBuilder::new(user)
    .field("username")
    .field("email")
    .execute(pool)
    .await?;

// 示例 3: 插入时忽略某些字段
let user = User { username: "test".to_string(), email: "test@example.com".to_string(), created_at: Some(now), ..Default::default() };
let id = InsertBuilder::new(user)
    .ignore_field("created_at")  // 让数据库使用默认值
    .execute(pool)
    .await?;
```

### 4.3 DeleteBuilder 使用示例

```rust
// 示例 1: 根据 WHERE 条件删除
let affected = DeleteBuilder::<User>::new()
    .condition(|b| b.and_eq("status", "deleted").and_lt("created_at", cutoff_date))
    .execute(pool)
    .await?;

// 示例 2: 删除所有记录（危险操作，可能需要额外确认）
let affected = DeleteBuilder::<User>::new()
    .execute(pool)
    .await?;

// 示例 3: 复杂 WHERE 条件
let affected = DeleteBuilder::<User>::new()
    .condition(|b| b
        .and_group(|g| g.and_eq("status", "inactive").and_lt("updated_at", cutoff))
        .or_eq("deleted_at", None)
    )
    .execute(pool)
    .await?;
```

## 5. 实现细节

### 5.1 UpdateBuilder 实现要点

1. **字段选择逻辑**：

   - 如果 `fields` 为空，更新所有非主键字段
   - 如果 `fields` 不为空，只更新指定的字段
   - 主键字段始终不参与更新

2. **WHERE 条件处理**：

   - 如果提供了 `where_builder`，使用其构建 WHERE 子句
   - 如果没有提供，默认使用主键条件：`WHERE pk = ?`

3. **SQL 生成**：

   ```sql
   UPDATE table_name
   SET field1 = ?, field2 = ?, ...
   WHERE conditions
   ```

4. **参数绑定**：
   - 先绑定 SET 子句的值
   - 再绑定 WHERE 子句的值

### 5.2 InsertBuilder 实现要点

1. **字段选择逻辑**：

   - 如果 `fields` 为空，插入所有非主键、非自动递增字段
   - 如果 `fields` 不为空，只插入指定的字段
   - 自动排除 `ignore_fields` 中的字段

2. **SQL 生成**：

   ```sql
   INSERT INTO table_name (field1, field2, ...)
   VALUES (?, ?, ...)
   ```

3. **返回值处理**：
   - MySQL: 使用 `LAST_INSERT_ID()`
   - PostgreSQL: 使用 `RETURNING id`
   - SQLite: 使用 `last_insert_rowid()`

### 5.3 DeleteBuilder 实现要点

1. **WHERE 条件处理**：

   - 如果提供了 `where_builder`，使用其构建 WHERE 子句
   - 如果没有提供，需要警告或抛出错误（防止误删所有数据）

2. **SQL 生成**：

   ```sql
   DELETE FROM table_name
   WHERE conditions
   ```

3. **安全考虑**：
   - 如果没有 WHERE 条件，可能需要额外的安全检查
   - 考虑添加 `allow_delete_all()` 方法用于明确允许删除所有记录

## 6. 与现有 API 的集成

### 6.1 扩展 Model Trait

```rust
pub trait Model: Sized {
    // ... 现有方法 ...

    /// 创建 UpdateBuilder
    fn update_builder(self) -> UpdateBuilder<Self> {
        UpdateBuilder::new(self)
    }

    /// 创建 InsertBuilder
    fn insert_builder(self) -> InsertBuilder<Self> {
        InsertBuilder::new(self)
    }
}

/// 为 Model 实现 DeleteBuilder 的便捷方法
impl<M: Model> DeleteBuilder<M> {
    /// 从 Model 创建 DeleteBuilder（类型推断辅助）
    pub fn from_model(_model: &M) -> Self {
        Self::new()
    }
}
```

### 6.2 使用方式

```rust
// 方式 1: 直接使用 Builder
let affected = UpdateBuilder::new(user)
    .field("username")
    .condition(|b| b.and_eq("id", 1))
    .execute(pool)
    .await?;

// 方式 2: 通过 Model 方法
let affected = user.update_builder()
    .field("username")
    .condition(|b| b.and_eq("id", 1))
    .execute(pool)
    .await?;

// 方式 3: DeleteBuilder（静态方法）
let affected = DeleteBuilder::<User>::new()
    .condition(|b| b.and_eq("status", "deleted"))
    .execute(pool)
    .await?;
```

## 7. 错误处理

### 7.1 验证错误

- **UpdateBuilder**:

  - 如果指定的字段不存在，返回错误
  - 如果 WHERE 条件为空且主键也为空，返回错误

- **InsertBuilder**:

  - 如果指定的字段不存在，返回错误
  - 如果必填字段未提供，返回错误

- **DeleteBuilder**:
  - 如果 WHERE 条件为空，返回错误（除非明确允许）

### 7.2 数据库错误

- 处理数据库约束错误
- 处理外键约束错误
- 提供友好的错误信息

## 8. 性能考虑

1. **字段验证**: 在构建时验证字段名，避免运行时错误
2. **SQL 缓存**: 考虑缓存常用的 SQL 语句
3. **批量操作**: 未来可以扩展支持批量更新/插入

## 9. 扩展性设计

### 9.1 未来可能的扩展

1. **批量操作**:

   ```rust
   UpdateBuilder::batch(users)
       .field("status")
       .condition(|b| b.and_in("id", ids))
       .execute(pool)
       .await?;
   ```

2. **条件更新**:

   ```rust
   UpdateBuilder::new(user)
       .field("status")
       .set_if("status", "active", |u| u.is_vip())
       .execute(pool)
       .await?;
   ```

3. **返回更新后的数据**:
   ```rust
   let updated = UpdateBuilder::new(user)
       .field("username")
       .returning()  // PostgreSQL 支持
       .execute(pool)
       .await?;
   ```

## 10. 实现优先级

### Phase 1: 核心功能

1. ✅ UpdateBuilder - 指定字段和 WHERE 条件
2. ✅ InsertBuilder - 指定插入字段
3. ✅ DeleteBuilder - 指定 WHERE 条件

### Phase 2: 增强功能

4. 字段验证和错误处理
5. 与现有 API 的集成
6. 文档和示例

### Phase 3: 高级功能

7. 批量操作支持
8. 条件更新
9. 性能优化

## 11. 文件结构

```
core/src/
  ├── crud_builder.rs      # 新增：CRUD Builder 实现
  ├── update_builder.rs    # UpdateBuilder 实现
  ├── insert_builder.rs    # InsertBuilder 实现
  ├── delete_builder.rs    # DeleteBuilder 实现
  └── traits.rs            # 扩展 Model trait
```

## 12. 测试策略

1. **单元测试**: 测试 SQL 生成逻辑
2. **集成测试**: 测试与数据库的交互
3. **边界测试**: 测试空字段、空条件等边界情况
4. **错误测试**: 测试各种错误场景

---

## 总结

这个设计方案提供了一个通用、灵活、类型安全的 CRUD Builder 系统，可以：

- 解决当前 Update 必须更新所有非 Option 字段的问题
- 支持灵活的字段选择
- 复用现有的 QueryBuilder WHERE 条件能力
- 为未来的扩展（批量操作、条件更新等）打下基础
