# sqlxplus

> 在保持 SQLx 性能与 SQL 灵活性的前提下，为 Rust 项目提供一套可生产、跨 MySQL/Postgres/SQLite 的高级数据库封装（CRUD、分页、动态查询、代码生成）。

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
sqlxplus = { version = "0.1.0", features = ["mysql"] }
sqlx = { version = "0.8.6", features = ["runtime-tokio-native-tls", "chrono", "mysql"] }
```

### 使用示例

```rust
use sqlxplus::{DbPool, Crud, QueryBuilder, ModelMeta, CRUD};

#[derive(Debug, sqlx::FromRow, ModelMeta, CRUD)]
#[model(table = "users", pk = "id")]
struct User {
    id: i64,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = DbPool::connect("mysql://user:pass@localhost/db").await?;

    // 查找用户
    let user = User::find_by_id(&pool, 1).await?;

    // 分页查询
    let builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1")
        .and_eq("status", "active")
        .order_by("created_at", false);
    let page = User::paginate(&pool, builder, 1, 10).await?;

    Ok(())
}
```

## 项目结构

```
sqlx-plus/
├─ core/               # 核心库（sqlxplus）- 已发布到 crates.io
├─ derive/             # proc-macro crate（sqlxplus-derive）- 已发布到 crates.io
├─ cli/                # 代码生成器
└─ examples/           # 示例项目
```

## 功能特性

- ✅ CRUD 操作（Create, Read, Update, Delete）
- ✅ 逻辑删除支持（soft delete）
- ✅ 分页查询（paginate）
- ✅ 安全查询构建器（QueryBuilder）
- ✅ 条件分组（AND/OR with parentheses）
- ✅ GROUP BY 和 HAVING 支持
- ✅ LIMIT/OFFSET 支持
- ✅ 多数据库支持（MySQL, PostgreSQL, SQLite）

## License

MIT OR Apache-2.0
