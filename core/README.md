# sqlxplus

[English](#english) | [中文](#中文)

---

<a name="english"></a>

> A production-ready, cross-database (MySQL / PostgreSQL / SQLite) advanced database toolkit for Rust — providing CRUD operations, pagination, dynamic query building, CRUD builders, and code generation — all while preserving SQLx's native performance and SQL flexibility.

[![Crates.io](https://img.shields.io/crates/v/sqlxplus.svg)](https://crates.io/crates/sqlxplus)
[![Documentation](https://docs.rs/sqlxplus/badge.svg)](https://docs.rs/sqlxplus)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Features

- **Cross-Database**: Supports MySQL, PostgreSQL, and SQLite — switch by changing the connection URL
- **Zero-Cost Abstractions**: All operations use native SQLx commands, no runtime abstraction overhead
- **Developer Experience**: ORM-like APIs (`Model` trait, `derive(CRUD)` macro, `QueryBuilder`), minimizing boilerplate
- **Extensible**: Custom SQL, transactions, raw query access; easy to extend to new databases
- **Type-Safe**: Parameterized SQL, compile-time checks wherever possible, no string concatenation for user inputs
- **Code Generation**: CLI tool to auto-generate models + CRUD + tests from database schemas

## Quick Start

### Installation

```toml
[dependencies]
sqlxplus = { version = "0.2.7", features = ["mysql"] }
sqlx = { version = "0.8.6", features = ["runtime-tokio-native-tls", "chrono", "mysql"] }
tokio = { version = "1.40", features = ["full"] }
```

Choose features based on your database:

| Database   | Feature      |
|-----------|-------------|
| MySQL      | `"mysql"`    |
| PostgreSQL | `"postgres"` |
| SQLite     | `"sqlite"`   |

You can also enable multiple databases simultaneously: `features = ["mysql", "postgres", "sqlite"]`

### Basic Example

```rust
use sqlxplus::{DbPool, Crud, QueryBuilder};

// Define a model
#[derive(Debug, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "users", pk = "id")]
struct User {
    id: Option<i64>,
    name: Option<String>,
    email: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = DbPool::connect("mysql://user:pass@localhost/db").await?;

    // Create
    let user = User {
        id: None,
        name: Some("Alice".to_string()),
        email: Some("alice@example.com".to_string()),
    };
    let id = user.insert(pool.mysql_pool()).await?;

    // Read
    let user = User::find_by_id(pool.mysql_pool(), id).await?;

    // Update
    if let Some(mut user) = user {
        user.name = Some("Bob".to_string());
        user.update(pool.mysql_pool()).await?;
    }

    // Delete
    User::delete_by_id(pool.mysql_pool(), id).await?;

    Ok(())
}
```

## Architecture

```
sqlx-plus/
├─ core/               # Core library (sqlxplus) — published on crates.io
│  └─ src/
│     ├─ traits.rs         # Model + Crud trait definitions
│     ├─ crud.rs           # Generic CRUD implementations (find, insert, update, delete, paginate)
│     ├─ builder/          # Query & CRUD Builder system
│     │  ├─ query_builder.rs   # Dynamic WHERE clause builder
│     │  ├─ update_builder.rs  # Selective field update builder
│     │  ├─ insert_builder.rs  # Selective field insert builder
│     │  └─ delete_builder.rs  # Conditional delete builder
│     ├─ db_pool.rs        # Unified connection pool (DbPool)
│     ├─ transaction.rs    # Transaction management (flat + nested via savepoints)
│     ├─ database_info.rs  # DB-specific info abstraction (placeholder, identifier escaping)
│     ├─ database_type.rs  # Automatic DB type inference from Pool/Transaction
│     ├─ executor.rs       # DbExecutor trait for pool/transaction unification
│     ├─ macros_api.rs     # Metadata structs used by proc-macros (FieldMeta, ModelMeta)
│     ├─ error.rs          # Error types (SqlxPlusError)
│     └─ utils.rs          # Utility functions
├─ derive/             # Proc-macro crate (sqlxplus-derive) — published on crates.io
│  └─ src/lib.rs           # #[derive(ModelMeta)] and #[derive(CRUD)] macros
├─ cli/                # Code generator (sqlxplus-cli) — published on crates.io
│  └─ src/
│     ├─ main.rs           # CLI entry point (generate / sql commands)
│     ├─ database.rs       # DB schema introspection
│     ├─ generator.rs      # Rust model code generator
│     └─ sql_generator.rs  # SQL DDL generator from Rust models
└─ examples/           # Example projects
   ├─ mysql_example/
   ├─ postgres_example/
   ├─ sqlite_example/
   └─ test_models/
```

## Detailed Documentation

### 1. Defining Models

Use `ModelMeta` and `CRUD` derive macros to auto-generate CRUD operations:

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

**Model attributes:**

| Attribute      | Description                                | Required |
|---------------|--------------------------------------------|----------|
| `table`       | Database table name                         | ✅ Yes   |
| `pk`          | Primary key field name (default: `"id"`)    | No       |
| `soft_delete` | Soft delete field name                      | No       |
| `table_comment` | Table comment (for SQL generation)       | No       |

**Field attributes** (via `#[column(...)]`):

| Attribute        | Description                                           |
|-----------------|-------------------------------------------------------|
| `primary_key`   | Marks as primary key                                  |
| `auto_increment`| Auto-increment field                                  |
| `not_null`      | NOT NULL constraint                                   |
| `default`       | Default value (SQL expression, e.g. `"0"`, `"CURRENT_TIMESTAMP"`) |
| `length`        | Field length (for VARCHAR, etc.)                      |
| `unique`        | Unique constraint                                     |
| `index`         | Creates an index on this field                        |
| `combine_index` | Combined index, format: `"idx_name:order"`            |
| `soft_delete`   | Marks as soft delete field                            |
| `comment`       | Column comment                                        |

### 2. Soft Delete

Enable soft delete by specifying the `soft_delete` attribute:

```rust
#[derive(Debug, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "posts", pk = "id", soft_delete = "is_deleted")]
struct Post {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub is_deleted: Option<i32>, // 0 = not deleted, 1 = deleted
}

// Soft delete (sets is_deleted = 1)
Post::delete_by_id(pool.mysql_pool(), 1).await?;

// Queries automatically filter deleted records
let post = Post::find_by_id(pool.mysql_pool(), 1).await?; // Returns None

// Force hard delete
Post::hard_delete_by_id(pool.mysql_pool(), 1).await?;
```

### 3. CRUD Operations

#### Create

```rust
let user = User {
    id: None,
    name: Some("Alice".to_string()),
    email: Some("alice@example.com".to_string()),
    status: Some(1),
};
let id = user.insert(pool.mysql_pool()).await?;
```

#### Read

```rust
// Find by ID
let user = User::find_by_id(pool.mysql_pool(), 1).await?;

// Find by multiple IDs
let users = User::find_by_ids(pool.mysql_pool(), vec![1, 2, 3]).await?;

// Find one with QueryBuilder
let builder = QueryBuilder::new("").and_eq("email", "alice@example.com");
let user = User::find_one(pool.mysql_pool(), builder).await?;

// Find all (max 1000 records)
let users = User::find_all(pool.mysql_pool(), None).await?;
```

#### Update

```rust
// Patch semantics: None fields are NOT updated (preserves DB values)
let mut user = User::find_by_id(pool.mysql_pool(), 1).await?.unwrap();
user.name = Some("Bob".to_string());
user.email = None; // email will NOT be changed
user.update(pool.mysql_pool()).await?;

// Reset semantics: None fields are reset to database defaults
user.update_with_none(pool.mysql_pool()).await?;
```

#### Delete

```rust
// Auto-select hard/soft delete based on model configuration
User::delete_by_id(pool.mysql_pool(), 1).await?;

// Force hard delete
User::hard_delete_by_id(pool.mysql_pool(), 1).await?;

// Force soft delete (requires soft_delete configuration)
User::soft_delete_by_id(pool.mysql_pool(), 1).await?;
```

### 4. QueryBuilder

`QueryBuilder` provides safe, flexible dynamic query construction:

```rust
use sqlxplus::QueryBuilder;

// Basic query
let builder = QueryBuilder::new("")
    .and_eq("status", 1)
    .and_like("name", "%Alice%")
    .order_by("created_at", false); // false = DESC

let users = User::find_all(pool.mysql_pool(), Some(builder)).await?;

// Condition grouping
let builder = QueryBuilder::new("")
    .and_group(|b| {
        b.or_eq("status", 1).or_eq("status", 2)
    })
    .and_gt("age", 18);
// SQL: WHERE (status = 1 OR status = 2) AND age > 18

// Complex query
let builder = QueryBuilder::new("")
    .and_in("category", vec!["tech", "news"])
    .and_between("price", 100, 500)
    .and_is_not_null("published_at")
    .order_by("views", false)
    .limit(20)
    .offset(40);
```

**Available methods:**

| Category    | Methods                                                     |
|------------|-------------------------------------------------------------|
| Comparison | `and_eq`, `and_ne`, `and_gt`, `and_ge`, `and_lt`, `and_le` |
| OR variants | `or_eq`, `or_ne`, `or_gt`, `or_ge`, `or_lt`, `or_le`      |
| Pattern    | `and_like`, `and_like_prefix`, `and_like_suffix`, `and_like_exact`, `and_like_custom`, `or_like` |
| Range      | `and_in`, `and_not_in`, `or_in`, `and_between`, `or_between` |
| Null       | `and_is_null`, `and_is_not_null`, `or_is_null`, `or_is_not_null` |
| Grouping   | `and_group`, `or_group`                                     |
| Aggregation| `group_by`, `having_eq`, `having_ne`, `having_gt`, `having_ge`, `having_lt`, `having_le` |
| Sorting    | `order_by`                                                  |
| Limit      | `limit`, `offset`                                           |

### 5. CRUD Builders

For advanced insert/update/delete scenarios beyond simple CRUD, use the Builder pattern:

#### UpdateBuilder — Selective Field Updates

```rust
use sqlxplus::UpdateBuilder;

// Update only specific fields
let user = User { id: Some(1), name: Some("NewName".to_string()), ..Default::default() };
let affected = UpdateBuilder::new(user)
    .field("name")
    .condition(|b| b.and_eq("id", 1))
    .execute(pool.mysql_pool())
    .await?;
```

#### InsertBuilder — Selective Field Inserts

```rust
use sqlxplus::InsertBuilder;

// Insert only specified fields
let user = User { name: Some("Alice".to_string()), email: Some("alice@example.com".to_string()), ..Default::default() };
let id = InsertBuilder::new(user)
    .field("name")
    .field("email")
    .execute(pool.mysql_pool())
    .await?;
```

#### DeleteBuilder — Conditional Deletes

```rust
use sqlxplus::DeleteBuilder;

// Delete with WHERE conditions
let affected = DeleteBuilder::<User>::new()
    .condition(|b| b.and_eq("status", 0).and_lt("created_at", cutoff_date))
    .execute(pool.mysql_pool())
    .await?;
```

### 6. Pagination

```rust
let builder = QueryBuilder::new("")
    .and_eq("status", 1)
    .order_by("created_at", false);

let page = User::paginate(pool.mysql_pool(), builder, 1, 10).await?;

println!("Total: {}", page.total);
println!("Page: {}", page.page);
println!("Size: {}", page.size);
println!("Pages: {}", page.pages);
println!("Items: {:?}", page.items);
```

### 7. Transactions

```rust
use sqlxplus::Transaction;

// Basic transaction
let mut tx = Transaction::begin(&pool).await?;
let id = user.insert(tx.as_mysql_executor()).await?;
let mut user = User::find_by_id(tx.as_mysql_executor(), id).await?.unwrap();
user.status = Some(2);
user.update(tx.as_mysql_executor()).await?;
tx.commit().await?;

// Callback-style transaction (auto commit/rollback)
use sqlxplus::with_transaction;
let result = with_transaction(&pool, |tx| Box::pin(async move {
    let id = user.insert(tx.as_mysql_executor()).await?;
    Ok(id)
})).await?;
```

**Nested transactions** (via savepoints):

```rust
use sqlxplus::{with_transaction, with_mysql_nested_transaction};

with_transaction(&pool, |tx| Box::pin(async move {
    // outer transaction work...
    
    with_mysql_nested_transaction(tx, |tx| Box::pin(async move {
        // nested work (uses SAVEPOINT)
        // rollback here only rolls back to the savepoint
        Ok(())
    })).await?;
    
    Ok(())
})).await?;
```

### 8. Database Connection

```rust
use sqlxplus::DbPool;

// MySQL
let pool = DbPool::connect("mysql://user:pass@localhost:3306/database").await?;

// PostgreSQL
let pool = DbPool::connect("postgres://user:pass@localhost:5432/database").await?;

// SQLite
let pool = DbPool::connect("sqlite://database.db").await?;
let pool = DbPool::connect("sqlite::memory:").await?; // In-memory database
```

### 9. Count

```rust
let builder = QueryBuilder::new("").and_eq("status", 1);
let count = User::count(pool.mysql_pool(), builder).await?;
```

## CLI Tool — `sqlxplus-cli`

A bidirectional code generator: **Database → Rust Model** and **Rust Model → SQL DDL**.

### Installation

```bash
cargo install sqlxplus-cli
```

### Generate Rust Models from Database

```bash
# Interactive table selection
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname"

# Generate all tables
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname" --all

# Generate specific tables to a directory
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname" -t users -t orders -o src/models

# Preview generated code (dry run)
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname" --dry-run
```

### Generate SQL from Rust Models

```bash
# Generate MySQL DDL from a model file
sqlxplus-cli sql -m src/models/user.rs -d mysql -o sql/user.sql

# Scan directory and generate SQL for all models
sqlxplus-cli sql -D src/models -d postgres -o sql/all_tables.sql
```

See the full [CLI documentation](cli/README.md) for details on options, type mappings, and advanced usage.

## Feature Checklist

- ✅ CRUD operations (Create, Read, Update, Delete)
- ✅ Soft delete support
- ✅ Pagination (`paginate`)
- ✅ Transaction support (flat + nested via savepoints)
- ✅ Safe QueryBuilder (parameterized, no SQL injection)
- ✅ Condition grouping (AND/OR with parentheses, nested)
- ✅ GROUP BY & HAVING support
- ✅ LIMIT / OFFSET
- ✅ Multi-database support (MySQL, PostgreSQL, SQLite)
- ✅ Type-safe parameter binding
- ✅ Compile-time type checks
- ✅ Async operations
- ✅ CRUD Builders (UpdateBuilder, InsertBuilder, DeleteBuilder)
- ✅ Bidirectional code generation (DB → Rust, Rust → SQL)

## Important Notes

1. **Field Types**: Use `Option<T>` wrapper fields to support NULL values and flexible update semantics
2. **Primary Key**: PK fields should typically be `Option<i64>` — set to `None` on insert for auto-generation
3. **Update Semantics**:
   - `update()`: **Patch** — `None` fields are skipped (DB values preserved)
   - `update_with_none()`: **Reset** — `None` fields are reset to DB defaults
4. **Performance**: QueryBuilder uses parameterized queries, preventing SQL injection with performance comparable to hand-written SQL
5. **DB Type Inference**: The database type is automatically inferred from the Pool/Transaction passed to each operation — no explicit type parameters needed

## Examples

See the `examples/` directory for complete working examples:

- [MySQL Example](examples/mysql_example/src/main.rs)
- [PostgreSQL Example](examples/postgres_example/src/main.rs)
- [SQLite Example](examples/sqlite_example/src/main.rs)

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a Pull Request

## License

MIT OR Apache-2.0

---

<a name="中文"></a>

# sqlxplus（中文文档）

> 在保持 SQLx 性能与 SQL 灵活性的前提下，为 Rust 项目提供一套可生产、跨 MySQL/Postgres/SQLite 的高级数据库封装（CRUD、分页、动态查询、CRUD Builder、代码生成）。

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
sqlxplus = { version = "0.2.7", features = ["mysql"] }
sqlx = { version = "0.8.6", features = ["runtime-tokio-native-tls", "chrono", "mysql"] }
tokio = { version = "1.40", features = ["full"] }
```

根据你使用的数据库选择对应的 feature：

| 数据库     | Feature      |
|-----------|-------------|
| MySQL      | `"mysql"`    |
| PostgreSQL | `"postgres"` |
| SQLite     | `"sqlite"`   |

或同时启用多个: `features = ["mysql", "postgres", "sqlite"]`

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
    let pool = DbPool::connect("mysql://user:pass@localhost/db").await?;

    // 插入
    let user = User {
        id: None,
        name: Some("张三".to_string()),
        email: Some("zhangsan@example.com".to_string()),
    };
    let id = user.insert(pool.mysql_pool()).await?;

    // 查询
    let user = User::find_by_id(pool.mysql_pool(), id).await?;

    // 更新
    if let Some(mut user) = user {
        user.name = Some("李四".to_string());
        user.update(pool.mysql_pool()).await?;
    }

    // 删除
    User::delete_by_id(pool.mysql_pool(), id).await?;

    Ok(())
}
```

## 架构概览

```
sqlx-plus/
├─ core/               # 核心库（sqlxplus）- 已发布到 crates.io
│  └─ src/
│     ├─ traits.rs         # Model + Crud trait 定义
│     ├─ crud.rs           # 泛型 CRUD 实现（find, insert, update, delete, paginate）
│     ├─ builder/          # 查询 & CRUD Builder 系统
│     │  ├─ query_builder.rs   # 动态 WHERE 条件构建器
│     │  ├─ update_builder.rs  # 选择性字段更新构建器
│     │  ├─ insert_builder.rs  # 选择性字段插入构建器
│     │  └─ delete_builder.rs  # 条件删除构建器
│     ├─ db_pool.rs        # 统一连接池（DbPool）
│     ├─ transaction.rs    # 事务管理（平级 + 嵌套 savepoint）
│     ├─ database_info.rs  # 数据库特性抽象（占位符、标识符转义）
│     ├─ database_type.rs  # 从 Pool/Transaction 自动推断数据库类型
│     ├─ executor.rs       # DbExecutor trait，统一 Pool 与 Transaction
│     ├─ macros_api.rs     # 宏使用的元数据结构体（FieldMeta, ModelMeta）
│     ├─ error.rs          # 错误类型（SqlxPlusError）
│     └─ utils.rs          # 工具函数
├─ derive/             # proc-macro crate（sqlxplus-derive）- 已发布到 crates.io
│  └─ src/lib.rs           # #[derive(ModelMeta)] 和 #[derive(CRUD)] 宏实现
├─ cli/                # 代码生成器（sqlxplus-cli）- 已发布到 crates.io
│  └─ src/
│     ├─ main.rs           # CLI 入口（generate / sql 命令）
│     ├─ database.rs       # 数据库 schema 自省
│     ├─ generator.rs      # Rust model 代码生成器
│     └─ sql_generator.rs  # 从 Rust model 生成 SQL DDL
└─ examples/           # 示例项目
   ├─ mysql_example/
   ├─ postgres_example/
   ├─ sqlite_example/
   └─ test_models/
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

**模型属性：** `table`（表名，必填）、`pk`（主键，默认 `"id"`）、`soft_delete`（逻辑删除字段）、`table_comment`（表注释）

**字段属性** `#[column(...)]`：`primary_key`, `auto_increment`, `not_null`, `default`, `length`, `unique`, `index`, `combine_index`, `soft_delete`, `comment`

### 2. 逻辑删除

```rust
#[derive(Debug, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "posts", pk = "id", soft_delete = "is_deleted")]
struct Post {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub is_deleted: Option<i32>, // 0=未删除，1=已删除
}

Post::delete_by_id(pool.mysql_pool(), 1).await?; // 将 is_deleted 设置为 1
let post = Post::find_by_id(pool.mysql_pool(), 1).await?; // 返回 None
Post::hard_delete_by_id(pool.mysql_pool(), 1).await?; // 强制物理删除
```

### 3. CRUD 操作

#### 插入（Create）

```rust
let user = User { id: None, name: Some("张三".to_string()), email: Some("zhangsan@example.com".to_string()), status: Some(1) };
let id = user.insert(pool.mysql_pool()).await?;
```

#### 查询（Read）

```rust
let user = User::find_by_id(pool.mysql_pool(), 1).await?;                          // 根据 ID
let users = User::find_by_ids(pool.mysql_pool(), vec![1, 2, 3]).await?;              // 根据多个 ID
let builder = QueryBuilder::new("").and_eq("email", "zhangsan@example.com");
let user = User::find_one(pool.mysql_pool(), builder).await?;                       // 使用 QueryBuilder
let users = User::find_all(pool.mysql_pool(), None).await?;                         // 所有（最多 1000 条）
```

#### 更新（Update）

```rust
// Patch 语义：Option 字段为 None 时不更新
let mut user = User::find_by_id(pool.mysql_pool(), 1).await?.unwrap();
user.name = Some("李四".to_string());
user.email = None; // 不更新 email 字段
user.update(pool.mysql_pool()).await?;

// Reset 语义：Option 字段为 None 时重置为数据库默认值
user.update_with_none(pool.mysql_pool()).await?;
```

#### 删除（Delete）

```rust
User::delete_by_id(pool.mysql_pool(), 1).await?;       // 根据配置自动选择物理/逻辑删除
User::hard_delete_by_id(pool.mysql_pool(), 1).await?;   // 强制物理删除
User::soft_delete_by_id(pool.mysql_pool(), 1).await?;   // 强制逻辑删除
```

### 4. 查询构建器

```rust
use sqlxplus::QueryBuilder;

// 基础查询
let builder = QueryBuilder::new("")
    .and_eq("status", 1)
    .and_like("name", "%张%")
    .order_by("created_at", false); // false = DESC

// 条件分组
let builder = QueryBuilder::new("")
    .and_group(|b| b.or_eq("status", 1).or_eq("status", 2))
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

**可用方法：** 比较（`and_eq/or_eq`, `and_ne/or_ne`, `and_gt/or_gt`, `and_ge/or_ge`, `and_lt/or_lt`, `and_le/or_le`）、模糊（`and_like`, `and_like_prefix`, `and_like_suffix`, `and_like_exact`, `and_like_custom`, `or_like`）、范围（`and_in/or_in`, `and_not_in`, `and_between/or_between`）、空值（`and_is_null/or_is_null`, `and_is_not_null/or_is_not_null`）、分组（`and_group`, `or_group`）、聚合（`group_by`, `having_eq/ne/gt/ge/lt/le`）、排序（`order_by`）、限制（`limit`, `offset`）

### 5. CRUD Builder

提供更灵活的插入、更新、删除操作：

#### UpdateBuilder — 选择性字段更新

```rust
use sqlxplus::UpdateBuilder;

let user = User { id: Some(1), name: Some("新名字".to_string()), ..Default::default() };
let affected = UpdateBuilder::new(user)
    .field("name")                                    // 只更新 name 字段
    .condition(|b| b.and_eq("id", 1))                 // WHERE 条件
    .execute(pool.mysql_pool())
    .await?;
```

#### InsertBuilder — 选择性字段插入

```rust
use sqlxplus::InsertBuilder;

let user = User { name: Some("张三".to_string()), email: Some("z@e.com".to_string()), ..Default::default() };
let id = InsertBuilder::new(user)
    .field("name")
    .field("email")
    .execute(pool.mysql_pool())
    .await?;
```

#### DeleteBuilder — 条件删除

```rust
use sqlxplus::DeleteBuilder;

let affected = DeleteBuilder::<User>::new()
    .condition(|b| b.and_eq("status", 0))
    .execute(pool.mysql_pool())
    .await?;
```

### 6. 分页查询

```rust
let builder = QueryBuilder::new("").and_eq("status", 1).order_by("created_at", false);
let page = User::paginate(pool.mysql_pool(), builder, 1, 10).await?;
// page.total, page.page, page.size, page.pages, page.items
```

### 7. 事务支持

```rust
use sqlxplus::Transaction;

// 手动事务
let mut tx = Transaction::begin(&pool).await?;
let id = user.insert(tx.as_mysql_executor()).await?;
tx.commit().await?;

// 回调式事务（自动提交/回滚）
use sqlxplus::with_transaction;
let result = with_transaction(&pool, |tx| Box::pin(async move {
    let id = user.insert(tx.as_mysql_executor()).await?;
    Ok(id)
})).await?;

// 嵌套事务（通过 SAVEPOINT）
use sqlxplus::with_mysql_nested_transaction;
with_transaction(&pool, |tx| Box::pin(async move {
    // 外层事务操作...
    with_mysql_nested_transaction(tx, |tx| Box::pin(async move {
        // 内层操作（使用 SAVEPOINT），回滚只影响到 savepoint
        Ok(())
    })).await?;
    Ok(())
})).await?;
```

### 8. 数据库连接

```rust
use sqlxplus::DbPool;

let pool = DbPool::connect("mysql://user:pass@localhost:3306/database").await?;     // MySQL
let pool = DbPool::connect("postgres://user:pass@localhost:5432/database").await?;  // PostgreSQL
let pool = DbPool::connect("sqlite://database.db").await?;                         // SQLite
let pool = DbPool::connect("sqlite::memory:").await?;                              // 内存数据库
```

### 9. 统计查询

```rust
let builder = QueryBuilder::new("").and_eq("status", 1);
let count = User::count(pool.mysql_pool(), builder).await?;
```

## CLI 工具 — `sqlxplus-cli`

双向代码生成器：**数据库 → Rust Model** 和 **Rust Model → SQL DDL**。

```bash
cargo install sqlxplus-cli

# 从数据库生成 Rust Model
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname" --all -o src/models

# 从 Rust Model 生成 SQL DDL
sqlxplus-cli sql -D src/models -d mysql -o sql/all_tables.sql
```

详细用法请参阅 [CLI 文档](cli/README.md)。

## 功能特性

- ✅ CRUD 操作（Create, Read, Update, Delete）
- ✅ 逻辑删除支持（soft delete）
- ✅ 分页查询（paginate）
- ✅ 事务支持（平级 + 嵌套 savepoint）
- ✅ 安全查询构建器（QueryBuilder）
- ✅ 条件分组（AND/OR with parentheses，嵌套）
- ✅ GROUP BY 和 HAVING 支持
- ✅ LIMIT/OFFSET 支持
- ✅ 多数据库支持（MySQL, PostgreSQL, SQLite）
- ✅ 类型安全的参数绑定
- ✅ 编译期类型检查
- ✅ 异步操作
- ✅ CRUD Builder（UpdateBuilder, InsertBuilder, DeleteBuilder）
- ✅ 双向代码生成（DB → Rust, Rust → SQL）

## 注意事项

1. **字段类型**：建议使用 `Option<T>` 包装字段，以支持 NULL 值和灵活的更新语义
2. **主键**：主键字段通常使用 `Option<i64>`，插入时设为 `None` 自动生成
3. **更新语义**：
   - `update()`: Patch 语义，`None` 值的字段不更新
   - `update_with_none()`: Reset 语义，`None` 值的字段重置为默认值
4. **性能**：查询构建器使用参数化查询，避免 SQL 注入，性能与手写 SQL 相当
5. **数据库类型推断**：数据库类型从传入的 Pool/Transaction 自动推断，无需显式指定类型参数

## 示例代码

查看 `examples/` 目录获取完整的示例代码：

- [MySQL 示例](examples/mysql_example/src/main.rs)
- [PostgreSQL 示例](examples/postgres_example/src/main.rs)
- [SQLite 示例](examples/sqlite_example/src/main.rs)

## 贡献

1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature/my-feature`)
3. 提交修改 (`git commit -am 'Add new feature'`)
4. 推送分支 (`git push origin feature/my-feature`)
5. 创建 Pull Request

## License

MIT OR Apache-2.0
