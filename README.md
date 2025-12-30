# sqlxplus

> 在保持 SQLx 性能与 SQL 灵活性的前提下，为 Rust 项目提供一套可生产、跨 MySQL/Postgres/SQLite 的高级数据库封装（CRUD、分页、动态查询、代码生成）。

[![Crates.io](https://img.shields.io/crates/v/sqlxplus.svg)](https://crates.io/crates/sqlxplus)
[![Documentation](https://docs.rs/sqlxplus/badge.svg)](https://docs.rs/sqlxplus)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## 特性

- **兼容性**：支持 MySQL、Postgres、SQLite，切换仅需配置 URL
- **性能**：所有底层使用 SQLx 原生命令，避免运行时抽象开销
- **开发体验**：提供类似 ORM 的便捷 API（`Model` trait、`derive(CRUD)` 宏、QueryBuilder），减少样板代码
- **可扩展性**：支持自定义 SQL、事务、原生 query 访问；易于扩展新数据库
- **安全性**：SQL 参数化、编译期检查（尽可能），严禁字符串拼接用于用户输入
- **可生成**：命令行工具从 schema 自动生成 model + CRUD + tests

## 快速开始

### 安装

```toml
[dependencies]
sqlxplus = { version = "0.1.9", features = ["mysql"] }
sqlx = { version = "0.8.6", features = ["runtime-tokio-native-tls", "chrono", "mysql"] }
tokio = { version = "1.40", features = ["full"] }
```

根据你使用的数据库选择对应的 feature：

- MySQL: `features = ["mysql"]`
- PostgreSQL: `features = ["postgres"]`
- SQLite: `features = ["sqlite"]`
- 或同时启用多个: `features = ["mysql", "postgres", "sqlite"]`

### 基础示例

```rust
use sqlxplus::{DbPool, Crud, QueryBuilder};

// 定义模型
#[derive(Debug, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "users", pk = "id")]
struct User {
    id: Option<i64>,
    name: Option<String>,
    email: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 连接数据库
    let pool = DbPool::connect("mysql://user:pass@localhost/db").await?;

    // 插入数据
    let user = User {
        id: None,
        name: Some("张三".to_string()),
        email: Some("zhangsan@example.com".to_string()),
    };
    let id = user.insert(&pool).await?;
    println!("插入成功，ID: {}", id);

    // 查找用户
    let user = User::find_by_id(&pool, id).await?;
    println!("查找到用户: {:?}", user);

    // 更新用户
    if let Some(mut user) = user {
        user.name = Some("李四".to_string());
        user.update(&pool).await?;
        println!("更新成功");
    }

    // 删除用户
    User::delete_by_id(&pool, id).await?;
    println!("删除成功");

    Ok(())
}
```

## 详细使用文档

### 1. 定义模型

使用 `ModelMeta` 和 `CRUD` 宏自动生成 CRUD 操作：

```rust
#[derive(Debug, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "users", pk = "id")]
struct User {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub status: Option<i32>,
}
```

**属性说明：**

- `table`: 数据库表名
- `pk`: 主键字段名（默认为 "id"）
- `soft_delete`: 逻辑删除字段名（可选）

### 2. 逻辑删除

支持逻辑删除（软删除），只需指定 `soft_delete` 字段：

```rust
#[derive(Debug, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "posts", pk = "id", soft_delete = "is_deleted")]
struct Post {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_deleted: Option<i32>, // 0=未删除，1=已删除
}

// 使用逻辑删除
Post::delete_by_id(&pool, 1).await?; // 将 is_deleted 设置为 1
// 查询时自动过滤已删除的记录
let post = Post::find_by_id(&pool, 1).await?; // 返回 None
```

### 3. CRUD 操作

#### 插入（Create）

```rust
let user = User {
    id: None,
    name: Some("张三".to_string()),
    email: Some("zhangsan@example.com".to_string()),
    status: Some(1),
};
let id = user.insert(&pool).await?;
```

#### 查询（Read）

```rust
// 根据 ID 查询单条记录
let user = User::find_by_id(&pool, 1).await?;

// 根据多个 ID 查询
let users = User::find_by_ids(&pool, vec![1, 2, 3]).await?;

// 使用查询构建器查询单条
let builder = QueryBuilder::new("").and_eq("email", "zhangsan@example.com");
let user = User::find_one(&pool, builder).await?;

// 查询所有（最多 1000 条）
let users = User::find_all(&pool, None).await?;
```

#### 更新（Update）

```rust
// Patch 语义：Option 字段为 None 时不更新
let mut user = User::find_by_id(&pool, 1).await?.unwrap();
user.name = Some("李四".to_string());
user.email = None; // 不更新 email 字段
user.update(&pool).await?;

// Reset 语义：Option 字段为 None 时重置为数据库默认值
user.update_with_none(&pool).await?;
```

#### 删除（Delete）

```rust
// 根据模型配置自动选择物理删除或逻辑删除
User::delete_by_id(&pool, 1).await?;

// 强制物理删除
User::hard_delete_by_id(&pool, 1).await?;

// 强制逻辑删除（需要配置 soft_delete）
User::soft_delete_by_id(&pool, 1).await?;
```

### 4. 查询构建器

`QueryBuilder` 提供了安全、灵活的动态查询构建：

```rust
use sqlxplus::QueryBuilder;

// 基础查询
let builder = QueryBuilder::new("")
    .and_eq("status", 1)
    .and_like("name", "%张%")
    .order_by("created_at", false); // false = DESC

let users = User::find_all(&pool, Some(builder)).await?;

// 条件分组
let builder = QueryBuilder::new("")
    .and_group(|b| {
        b.or_eq("status", 1).or_eq("status", 2)
    })
    .and_gt("age", 18);
// SQL: WHERE (status = 1 OR status = 2) AND age > 18

// 复杂查询
let builder = QueryBuilder::new("")
    .and_in("category", vec!["tech", "news"])
    .and_between("price", 100, 500)
    .and_is_not_null("published_at")
    .order_by("views", false)
    .limit(20)
    .offset(40);
```

**可用方法：**

- 比较：`and_eq`, `and_ne`, `and_gt`, `and_gte`, `and_lt`, `and_lte`
- 模糊：`and_like`, `and_not_like`
- 范围：`and_in`, `and_not_in`, `and_between`
- 空值：`and_is_null`, `and_is_not_null`
- 分组：`and_group`, `or_group`
- 排序：`order_by`
- 限制：`limit`, `offset`

### 5. 分页查询

```rust
let builder = QueryBuilder::new("")
    .and_eq("status", 1)
    .order_by("created_at", false);

let page = User::paginate(&pool, builder, 1, 10).await?;

println!("总数: {}", page.total);
println!("当前页: {}", page.page);
println!("每页大小: {}", page.size);
println!("总页数: {}", page.pages);
println!("数据: {:?}", page.items);
```

### 6. 事务支持

```rust
use sqlxplus::Transaction;

// 开启事务
let mut tx = pool.begin().await?;

// 在事务中执行操作
let user = User {
    id: None,
    name: Some("张三".to_string()),
    email: Some("zhangsan@example.com".to_string()),
    status: Some(1),
};
let id = user.insert(&mut tx).await?;

// 更新
let mut user = User::find_by_id(&mut tx, id).await?.unwrap();
user.status = Some(2);
user.update(&mut tx).await?;

// 提交事务
tx.commit().await?;
```

### 7. 数据库连接

```rust
use sqlxplus::DbPool;

// MySQL
let pool = DbPool::connect("mysql://user:pass@localhost:3306/database").await?;

// PostgreSQL
let pool = DbPool::connect("postgres://user:pass@localhost:5432/database").await?;

// SQLite
let pool = DbPool::connect("sqlite://database.db").await?;
let pool = DbPool::connect("sqlite::memory:").await?; // 内存数据库
```

### 8. 统计查询

```rust
let builder = QueryBuilder::new("").and_eq("status", 1);
let count = User::count(&pool, builder).await?;
println!("符合条件的记录数: {}", count);
```

## 项目结构

```
sqlx-plus/
├─ core/               # 核心库（sqlxplus）- 已发布到 crates.io
├─ derive/             # proc-macro crate（sqlxplus-derive）- 已发布到 crates.io
├─ cli/                # 代码生成器
└─ examples/           # 示例项目
   ├─ mysql_example/
   ├─ postgres_example/
   └─ sqlite_example/
```

## 功能特性

- ✅ CRUD 操作（Create, Read, Update, Delete）
- ✅ 逻辑删除支持（soft delete）
- ✅ 分页查询（paginate）
- ✅ 事务支持（Transaction）
- ✅ 安全查询构建器（QueryBuilder）
- ✅ 条件分组（AND/OR with parentheses）
- ✅ GROUP BY 和 HAVING 支持
- ✅ LIMIT/OFFSET 支持
- ✅ 多数据库支持（MySQL, PostgreSQL, SQLite）
- ✅ 类型安全的参数绑定
- ✅ 编译期类型检查
- ✅ 异步操作

## 注意事项

1. **字段类型**：建议使用 `Option<T>` 包装字段，以支持 NULL 值和灵活的更新语义
2. **主键**：主键字段通常使用 `Option<i64>`，插入时设为 `None` 自动生成
3. **更新语义**：
   - `update()`: Patch 语义，`None` 值的字段不更新
   - `update_with_none()`: Reset 语义，`None` 值的字段重置为默认值
4. **性能**：查询构建器使用参数化查询，避免 SQL 注入，性能与手写 SQL 相当

## 示例代码

查看 `examples/` 目录获取完整的示例代码：

- [MySQL 示例](examples/mysql_example/src/main.rs)
- [PostgreSQL 示例](examples/postgres_example/src/main.rs)
- [SQLite 示例](examples/sqlite_example/src/main.rs)

## License

MIT OR Apache-2.0
