# sqlxplus-cli

sqlxplus 的代码生成工具，可以从数据库表结构自动生成 Rust Model 代码，也可以从 Rust Model 代码生成 CREATE TABLE SQL 语句。

## 安装

### 从源码安装

```bash
cargo install --path cli
```

### 从 crates.io 安装（待发布）

```bash
cargo install sqlxplus-cli
```

## 命令

CLI 工具提供两个主要命令：

1. **`generate`** - 从数据库表结构生成 Rust Model 代码
2. **`sql`** - 从 Rust Model 代码生成 CREATE TABLE SQL 语句

## 命令：generate

从数据库表结构自动生成 Rust Model 代码，包含完整的字段宏标注（索引、唯一约束、注释等）。

### 基本用法

```bash
# 交互式选择表
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname"

# 生成所有表
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname" --all

# 生成指定表
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname" -t users -t orders

# 指定输出目录
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname" -o src/models

# 覆盖已存在的文件
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname" --overwrite

# 预览生成的代码（不写入文件）
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname" --dry-run
```

### 支持的数据库

- **MySQL**: `mysql://user:pass@localhost/dbname`
- **PostgreSQL**: `postgres://user:pass@localhost/dbname`
  - 支持 `search_path` 参数：`postgres://user:pass@localhost/dbname?options=-csearch_path%3Dtest`
- **SQLite**: `sqlite://path/to/database.db` 或 `sqlite:path/to/database.db`

### 选项说明

- `-d, --database-url`: 数据库连接 URL（必需）
- `-o, --output`: 输出目录，默认为 `models`
- `-t, --tables`: 指定要生成的表名（可多次使用）
- `-a, --all`: 生成所有表，不进行交互式选择
- `--overwrite`: 覆盖已存在的文件
- `--dry-run`: 预览模式，不写入文件
- `--serde`: 生成 serde 序列化/反序列化 derives（默认启用）
- `--derive-crud`: 生成 CRUD derives（默认启用）

### 生成示例

假设数据库中有以下表：

```sql
CREATE TABLE users (
    id BIGINT PRIMARY KEY AUTO_INCREMENT COMMENT '主键ID',
    username VARCHAR(50) NOT NULL COMMENT '用户名',
    email VARCHAR(100) NOT NULL UNIQUE COMMENT '邮箱地址',
    is_del TINYINT DEFAULT 0 COMMENT '是否删除',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间'
) COMMENT '用户表';
```

运行命令：

```bash
sqlxplus-cli generate -d "mysql://user:pass@localhost/dbname" -t users
```

生成的代码：

```rust
#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "users", pk = "id", soft_delete = "is_del", table_comment = "用户表")]
pub struct Users {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    
    /// username (varchar(50)) | 非空
    #[column(not_null, length = 50, comment = "用户名")]
    pub username: Option<String>,
    
    /// email (varchar(100)) | 非空
    #[column(not_null, unique, index, length = 100, comment = "邮箱地址")]
    pub email: Option<String>,
    
    /// is_del (tinyint) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", soft_delete, comment = "是否删除")]
    pub is_del: Option<i16>,
    
    /// created_at (timestamp) | 可空
    /// 默认值: CURRENT_TIMESTAMP
    #[column(default = "CURRENT_TIMESTAMP", comment = "创建时间")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

### 生成的字段宏标注

从数据库逆向生成的代码会自动包含以下字段宏标注：

- **`primary_key`**: 主键字段
- **`auto_increment`**: 自增字段
- **`not_null`**: 非空字段
- **`default`**: 默认值
- **`length`**: 字段长度（VARCHAR 等类型）
- **`unique`**: 唯一索引字段
- **`index`**: 普通索引字段
- **`soft_delete`**: 逻辑删除字段（自动检测 `is_del`, `is_deleted`, `deleted_at` 等）
- **`comment`**: 字段注释（从数据库获取）

### 表注释支持

如果数据库表有注释，会自动生成到 `#[model(...)]` 属性中：

```rust
#[model(table = "users", pk = "id", soft_delete = "is_del", table_comment = "用户表")]
```

## 命令：sql

从 Rust Model 代码生成 CREATE TABLE SQL 语句，支持 MySQL、PostgreSQL 和 SQLite。

### 基本用法

```bash
# 生成 MySQL SQL
sqlxplus-cli sql -m src/models/user.rs -d mysql -o sql/user_mysql.sql

# 生成 PostgreSQL SQL
sqlxplus-cli sql -m src/models/user.rs -d postgres -o sql/user_postgres.sql

# 生成 SQLite SQL
sqlxplus-cli sql -m src/models/user.rs -d sqlite -o sql/user_sqlite.sql

# 输出到标准输出
sqlxplus-cli sql -m src/models/user.rs -d mysql
```

### 选项说明

- `-m, --model`: Rust Model 文件路径（必需）
- `-d, --database`: 数据库类型（`mysql`, `postgres`, `sqlite`），默认为 `mysql`
- `-o, --output`: 输出 SQL 文件路径（可选，不指定则输出到标准输出）

### 支持的字段宏标注

SQL 生成器支持以下字段宏标注：

- **`primary_key`**: 生成 PRIMARY KEY 约束
- **`auto_increment`**: 生成 AUTO_INCREMENT（MySQL）或 SERIAL（PostgreSQL）
- **`not_null`**: 生成 NOT NULL 约束
- **`default`**: 生成 DEFAULT 值
- **`length`**: 指定字段长度（如 VARCHAR(255)）
- **`unique`**: 生成唯一索引
- **`index`**: 生成普通索引
- **`combine_index`**: 生成联合索引（格式：`combine_index = "idx_name:order"`）
- **`soft_delete`**: 逻辑删除字段（不影响 SQL 生成）
- **`comment`**: 生成字段注释（数据库特定语法）

### SQL 生成示例

假设有以下 Rust Model：

```rust
#[derive(Debug, Default, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "user", pk = "id", soft_delete = "is_del", table_comment = "用户表")]
pub struct User {
    #[column(primary_key, auto_increment, comment = "主键ID")]
    pub id: Option<i64>,
    
    #[column(not_null, default = "1", comment = "系统类型")]
    pub system_type: Option<i16>,
    
    #[column(index, length = 255, comment = "用户名")]
    pub username: Option<String>,
    
    #[column(unique, index, length = 255, comment = "邮箱地址")]
    pub email: Option<String>,
    
    #[column(not_null, default = "0", index, soft_delete, comment = "是否删除")]
    pub is_del: Option<i16>,
}
```

生成的 MySQL SQL：

```sql
CREATE TABLE `user` (
    `id` BIGINT NOT NULL AUTO_INCREMENT COMMENT '主键ID',
    `system_type` TINYINT NOT NULL DEFAULT 1 COMMENT '系统类型',
    `username` VARCHAR(255) COMMENT '用户名',
    `email` VARCHAR(255) COMMENT '邮箱地址',
    `is_del` TINYINT NOT NULL DEFAULT 0 COMMENT '是否删除',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_user_email` (`email`)
) COMMENT '用户表';

CREATE INDEX `idx_user_username` ON `user` (`username`);
CREATE INDEX `idx_user_is_del` ON `user` (`is_del`);
```

生成的 PostgreSQL SQL：

```sql
CREATE TABLE "user" (
    "id" BIGSERIAL NOT NULL,
    "system_type" SMALLINT NOT NULL DEFAULT 1,
    "username" VARCHAR(255),
    "email" VARCHAR(255),
    "is_del" SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY ("id"),
    CONSTRAINT "uk_user_email" UNIQUE ("email")
);

CREATE INDEX "idx_user_username" ON "user" ("username");
CREATE INDEX "idx_user_is_del" ON "user" ("is_del");

COMMENT ON COLUMN "user"."id" IS '主键ID';
COMMENT ON COLUMN "user"."system_type" IS '系统类型';
COMMENT ON COLUMN "user"."username" IS '用户名';
COMMENT ON COLUMN "user"."email" IS '邮箱地址';
COMMENT ON COLUMN "user"."is_del" IS '是否删除';
COMMENT ON TABLE "user" IS '用户表';
```

## 特性

### 代码生成（generate 命令）

- ✅ 自动检测主键字段
- ✅ 自动检测逻辑删除字段（`is_del`, `is_deleted`, `deleted_at` 等）
- ✅ 支持 MySQL、PostgreSQL、SQLite
- ✅ 支持 PostgreSQL 的 `search_path` 参数
- ✅ 交互式表选择
- ✅ 批量生成多个表
- ✅ 自动生成 `mod.rs` 模块文件
- ✅ 类型映射（SQL 类型 → Rust 类型）
- ✅ 自动生成字段宏标注（索引、唯一约束、注释等）
- ✅ 支持表注释和字段注释
- ✅ 自动检测索引和唯一约束

### SQL 生成（sql 命令）

- ✅ 从 Rust Model 生成 CREATE TABLE SQL
- ✅ 支持 MySQL、PostgreSQL、SQLite 三种数据库
- ✅ 生成数据库特定的 SQL 语法
- ✅ 支持字段注释和表注释
- ✅ 支持单独索引和联合索引
- ✅ 支持唯一索引和普通索引
- ✅ 自动处理数据库类型差异

## 类型映射

### SQL → Rust 类型映射

| SQL 类型                | Rust 类型                        |
| ----------------------- | -------------------------------- |
| `BIGINT`, `BIGSERIAL`   | `i64`                            |
| `INT`, `INTEGER`, `SERIAL` | `i32`                          |
| `SMALLINT`, `TINYINT`, `SMALLSERIAL` | `i16`                    |
| `VARCHAR`, `TEXT`, `CHARACTER VARYING` | `String`                  |
| `DECIMAL`, `DOUBLE`, `NUMERIC` | `f64`                      |
| `BOOLEAN`, `BOOL`, `BIT` | `bool`                           |
| `DATE`                  | `chrono::NaiveDate`            |
| `DATETIME`              | `chrono::NaiveDateTime`         |
| `TIMESTAMP` (MySQL)     | `chrono::DateTime<chrono::Utc>` |
| `TIMESTAMP WITH TIME ZONE` (PostgreSQL) | `chrono::DateTime<chrono::Utc>` |
| `TIMESTAMP WITHOUT TIME ZONE` (PostgreSQL) | `chrono::NaiveDateTime` |
| `BLOB`, `BYTEA`         | `Vec<u8>`                        |
| `JSON`, `JSONB`         | `serde_json::Value`              |
| `UUID`                  | `uuid::Uuid`                     |

### 类型选择规则

- 如果字段为 `NULLable` 或有默认值，生成 `Option<T>`
- 如果字段为 `NOT NULL` 且无默认值，生成 `T`
- `String` 类型统一使用 `Option<String>` 以保持一致性

## 使用场景

### 场景 1：从数据库生成 Model 代码

适用于已有数据库表，需要快速生成对应的 Rust Model 代码：

```bash
# 从 PostgreSQL 数据库生成代码
sqlxplus-cli generate \
  -d "postgres://user:pass@localhost/dbname?options=-csearch_path%3Dtest" \
  -t users \
  -o src/models \
  --overwrite
```

### 场景 2：从 Model 代码生成 SQL

适用于先定义 Rust Model，然后生成数据库建表 SQL：

```bash
# 生成 MySQL 建表 SQL
sqlxplus-cli sql \
  -m src/models/user.rs \
  -d mysql \
  -o migrations/001_create_user.sql
```

### 场景 3：数据库迁移

结合使用两个命令，实现数据库迁移：

1. 从旧数据库生成 Model 代码
2. 修改 Model 代码
3. 生成新的 SQL 迁移脚本

## 注意事项

1. **PostgreSQL search_path**: 如果使用自定义 schema，需要在连接 URL 中指定：
   ```
   postgres://user:pass@localhost/dbname?options=-csearch_path%3Dyour_schema
   ```

2. **字段注释**: 
   - MySQL: 注释直接包含在 CREATE TABLE 语句中
   - PostgreSQL: 使用 `COMMENT ON` 语句单独添加
   - SQLite: 不支持注释，但会在生成的 SQL 中使用 SQL 注释（`--`）

3. **索引生成**:
   - 唯一索引会同时生成 `unique` 和 `index` 属性
   - 联合索引使用 `combine_index = "idx_name:order"` 格式

4. **默认值处理**:
   - PostgreSQL 的序列（`nextval`）会被自动识别为 `auto_increment`
   - 空字符串默认值会正确处理

## 常见问题

### Q: 如何生成包含所有字段宏标注的代码？

A: 使用 `generate` 命令从数据库生成，会自动包含所有字段宏标注（索引、唯一约束、注释等）。

### Q: 生成的代码与手动编写的代码有什么区别？

A: 主要区别在于：
- 字段类型注释使用数据库实际类型名称（如 PostgreSQL 的 `character varying` vs MySQL 的 `varchar`）
- 属性顺序可能略有不同（不影响功能）

### Q: 如何支持自定义 schema？

A: 对于 PostgreSQL，在连接 URL 中使用 `search_path` 参数即可。

## License

MIT OR Apache-2.0
