# sqlxplus-cli

sqlxplus 的代码生成工具，可以从数据库表结构自动生成 Rust Model 代码。

## 安装

### 从源码安装

```bash
cargo install --path cli
```

### 从 crates.io 安装（待发布）

```bash
cargo install sqlxplus-cli
```

## 使用方法

### 基本用法

```bash
# 交互式选择表
sqlxplus-cli -d "mysql://user:pass@localhost/dbname"

# 生成所有表
sqlxplus-cli -d "mysql://user:pass@localhost/dbname" --all

# 生成指定表
sqlxplus-cli -d "mysql://user:pass@localhost/dbname" -t users -t orders

# 指定输出目录
sqlxplus-cli -d "mysql://user:pass@localhost/dbname" -o src/models

# 预览生成的代码（不写入文件）
sqlxplus-cli -d "mysql://user:pass@localhost/dbname" --dry-run
```

### 支持的数据库

- **MySQL**: `mysql://user:pass@localhost/dbname`
- **PostgreSQL**: `postgres://user:pass@localhost/dbname`
- **SQLite**: `sqlite://path/to/database.db` 或 `sqlite:path/to/database.db`

### 选项说明

- `-d, --database-url`: 数据库连接 URL（必需）
- `-o, --output`: 输出目录，默认为 `models`
- `-t, --tables`: 指定要生成的表名（可多次使用）
- `-a, --all`: 生成所有表，不进行交互式选择
- `--overwrite`: 覆盖已存在的文件
- `--dry-run`: 预览模式，不写入文件
- `--serde`: 生成 serde 序列化/反序列化 derives
- `--derive-crud`: 生成 CRUD derives（默认启用）

## 生成示例

假设数据库中有以下表：

```sql
CREATE TABLE users (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL,
    is_del TINYINT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

运行命令：

```bash
sqlxplus-cli -d "mysql://user:pass@localhost/dbname" -t users
```

生成的代码：

```rust
#[derive(Debug, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "users", pk = "id", soft_delete = "is_del")]
pub struct Users {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub is_del: i16,
    pub created_at: Option<chrono::NaiveDateTime>,
}
```

## 特性

- ✅ 自动检测主键字段
- ✅ 自动检测逻辑删除字段（`is_del`, `is_deleted`, `deleted_at` 等）
- ✅ 支持 MySQL、PostgreSQL、SQLite
- ✅ 交互式表选择
- ✅ 批量生成多个表
- ✅ 自动生成 `mod.rs` 模块文件
- ✅ 类型映射（SQL 类型 → Rust 类型）

## 类型映射

| SQL 类型                | Rust 类型               |
| ----------------------- | ----------------------- |
| `BIGINT`                | `i64`                   |
| `INT`, `INTEGER`        | `i32`                   |
| `SMALLINT`, `TINYINT`   | `i16` / `i8`            |
| `VARCHAR`, `TEXT`       | `String`                |
| `DECIMAL`, `DOUBLE`     | `f64`                   |
| `BOOLEAN`, `BOOL`       | `bool`                  |
| `DATE`                  | `chrono::NaiveDate`     |
| `DATETIME`, `TIMESTAMP` | `chrono::NaiveDateTime` |
| `BLOB`, `BYTEA`         | `Vec<u8>`               |
| `JSON`, `JSONB`         | `serde_json::Value`     |

## License

MIT OR Apache-2.0
