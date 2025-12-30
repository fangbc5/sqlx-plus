# CRUD 函数泛型化重构方案

## 一、目标

将当前使用宏生成三套数据库特定函数（`find_by_id_mysql`, `find_by_id_postgres`, `find_by_id_sqlite`）的方式，重构为使用泛型参数统一实现的方案，减少代码重复，提高可维护性。

## 二、当前问题分析

### 2.1 代码重复

- 每个 CRUD 函数需要为 MySQL、PostgreSQL、SQLite 生成三个版本
- 核心逻辑相同，只有数据库特定的细节不同（占位符、Row 类型、绑定方式）
- 维护成本高：修改逻辑需要同步修改三处

### 2.2 宏复杂度

- 大量宏定义使代码可读性降低
- 调试困难
- 编译错误信息不友好

### 2.3 API 冗余

- 用户需要记住 `find_by_id_mysql`、`find_by_id_postgres`、`find_by_id_sqlite`
- 方法名冗长
- 类型推断不够智能

## 三、技术方案

### 3.1 核心设计

使用 **泛型参数 + Database trait bound + 辅助 trait** 的方式：

1. **定义 `DatabaseInfo` trait**：抽象数据库差异
2. **使用 `sqlx::Database` trait**：利用 sqlx 的类型系统
3. **统一函数签名**：通过泛型参数指定数据库类型

### 3.2 架构设计

```
DatabaseInfo trait (抽象数据库差异)
    ├── placeholder(index) -> String
    ├── escape_identifier(name) -> String
    └── get_driver() -> DbDriver

统一 CRUD 函数
    ├── find_by_id<DB, M, E>(...) -> Result<Option<M>>
    ├── find_by_ids<DB, M, E>(...) -> Result<Vec<M>>
    ├── find_all<DB, M, E>(...) -> Result<Vec<M>>
    └── ... (其他 CRUD 函数)
```

### 3.3 关键实现点

#### 3.3.1 DatabaseInfo Trait

```rust
pub trait DatabaseInfo: sqlx::Database {
    /// 获取占位符（MySQL/SQLite: "?", PostgreSQL: "$1", "$2", ...）
    fn placeholder(index: usize) -> String;

    /// 转义标识符（MySQL: `name`, PostgreSQL/SQLite: "name"）
    fn escape_identifier(name: &str) -> String;

    /// 获取数据库驱动类型
    fn get_driver() -> DbDriver;

    /// 获取 Row 类型（通过关联类型）
    type Row: sqlx::Row;
}
```

#### 3.3.2 统一函数签名示例

```rust
pub async fn find_by_id<DB, M, E>(
    executor: E,
    id: impl for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync,
) -> Result<Option<M>>
where
    DB: sqlx::Database + DatabaseInfo,
    M: Model + for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
    E: sqlx::Executor<'c, Database = DB> + Send,
{
    // 统一实现逻辑
}
```

## 四、实施步骤

### 阶段 1：定义 DatabaseInfo trait 并实现（基础准备）

**目标**：创建数据库信息抽象层

**任务清单**：

- [x] 在 `core/src/crud.rs` 或新建 `core/src/database_info.rs` 中定义 `DatabaseInfo` trait
- [x] 为 `sqlx::MySql` 实现 `DatabaseInfo`
- [x] 为 `sqlx::Postgres` 实现 `DatabaseInfo`
- [x] 为 `sqlx::Sqlite` 实现 `DatabaseInfo`
- [x] 编写单元测试验证实现正确性

**预期产出**：

- `DatabaseInfo` trait 定义
- 三个数据库的实现
- 基础测试用例

**风险评估**：

- 低风险：主要是 trait 定义和简单实现
- 需要验证占位符和转义逻辑的正确性

---

### 阶段 2：重构 find_by_id 函数（试点验证）

**目标**：验证方案可行性，作为后续重构的模板

**任务清单**：

- [x] 创建泛型版本的 `find_by_id<DB, M, E>` 函数
- [x] 保留旧的数据库特定函数（`find_by_id_mysql` 等）作为兼容层
- [x] 在兼容层中调用新的泛型函数
- [ ] 更新 `Crud` trait 中的 `impl_find_by_id` 宏，调用新函数
- [ ] 运行所有测试确保兼容性
- [ ] 更新文档和示例

**代码示例**：

```rust
// 新的泛型实现
pub async fn find_by_id<'e, 'c: 'e, DB, M, E>(
    executor: E,
    id: impl for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync,
) -> Result<Option<M>>
where
    DB: Database + DatabaseInfo,
    for<'a> DB::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    M: Model + for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
    E: sqlx::Executor<'c, Database = DB> + Send,
{
    // 使用 DatabaseInfo trait 获取数据库特定信息
    let escaped_table = DB::escape_identifier(M::TABLE);
    let escaped_pk = DB::escape_identifier(M::PK);
    let placeholder = DB::placeholder(0);
    // ... 实现逻辑
}

// trait 中的方法直接调用泛型版本
// 不再需要兼容层函数（find_by_id_mysql 等已移除）
impl_find_by_id!("mysql", sqlx::MySql, find_by_id_mysql);
// 内部实现：crate::crud::find_by_id::<sqlx::MySql, Self, E>(executor, id).await
```

**预期产出**：

- ✅ 泛型版本的 `find_by_id` 函数
- ✅ 验证方案可行性
- ✅ 确定后续重构模式
- ✅ 移除冗余的兼容层函数，直接使用泛型版本
- ✅ 所有 examples 编译通过

**风险评估**：

- 中等风险：需要处理类型约束和绑定逻辑
- 需要仔细测试边界情况

---

### 阶段 3：逐步迁移其他查询函数

**目标**：将其他查询函数迁移到泛型实现

**迁移顺序**（按复杂度递增）：

1. `find_by_ids` - 相对简单
2. `find_one` - 需要处理 QueryBuilder
3. `find_all` - 需要处理 QueryBuilder 和绑定
4. `count` - 需要处理聚合查询
5. `paginate` - 最复杂，需要分页逻辑

**任务清单**（每个函数）：

- [ ] 创建泛型版本函数
- [ ] 保留兼容层（可选）
- [ ] 更新 trait 宏调用
- [ ] 运行测试
- [ ] 更新文档

**预期产出**：

- 所有查询函数都有泛型版本
- 保持向后兼容

**风险评估**：

- 中等风险：不同函数的复杂度不同
- `paginate` 函数可能最复杂

---

### 阶段 4：迁移删除函数

**目标**：迁移删除相关函数

**任务清单**：

- [ ] `hard_delete_by_id<DB, M, E>`
- [ ] `soft_delete_by_id<DB, M, E>`
- [ ] `delete_by_id<DB, M, E>`（内部调用 hard/soft）

**预期产出**：

- 删除函数泛型化完成

**风险评估**：

- 低风险：逻辑相对简单

---

### 阶段 5：更新 Crud trait 定义

**目标**：简化 trait 定义，使用泛型方法

**任务清单**：

- [ ] 分析当前 trait 方法签名
- [ ] 设计新的泛型方法签名
- [ ] 考虑向后兼容性（是否保留旧方法）
- [ ] 更新 trait 定义
- [ ] 更新 derive 宏以生成新签名的方法
- [ ] 更新所有使用 trait 的代码

**新的 trait 设计**：

```rust
pub trait Crud: Model + Send + Sync + Unpin {
    // 泛型方法
    async fn find_by_id<DB, E>(...) -> Result<Option<Self>>
    where
        DB: sqlx::Database + DatabaseInfo,
        Self: for<'r> sqlx::FromRow<'r, DB::Row>,
        E: sqlx::Executor<'c, Database = DB> + Send;

    // ... 其他方法
}
```

**预期产出**：

- 简化的 trait 定义
- 更新的 derive 宏
- 更新的示例代码

**风险评估**：

- 高风险：这是破坏性变更
- 需要仔细设计迁移策略
- 可能需要提供兼容层

---

### 阶段 6：清理和优化

**目标**：移除冗余代码，优化实现

**任务清单**：

- [ ] 移除旧的宏定义（如果不再需要）
- [ ] 移除兼容层函数（如果决定完全迁移）
- [ ] 优化代码结构
- [ ] 更新所有文档
- [ ] 更新示例代码
- [ ] 性能测试和优化
- [ ] 代码审查

**预期产出**：

- 干净的代码库
- 完整的文档
- 更新的示例

**风险评估**：

- 低风险：主要是清理工作

---

## 五、关键技术挑战

### 5.1 占位符处理

**问题**：MySQL/SQLite 使用 `?`，PostgreSQL 使用 `$1, $2, ...`

**解决方案**：

- 在 `DatabaseInfo::placeholder(index)` 中处理
- 对于 PostgreSQL，需要跟踪参数索引

### 5.2 Row 类型获取

**问题**：需要从 `DB` 泛型参数获取对应的 `Row` 类型

**解决方案**：

- 使用 `DB::Row`（sqlx::Database 的关联类型）
- 在 `DatabaseInfo` trait 中也可以定义关联类型

### 5.3 绑定值处理

**问题**：不同数据库的绑定方式可能不同

**解决方案**：

- 使用 sqlx 的统一绑定接口
- `query.bind()` 方法已经处理了数据库差异

### 5.4 QueryBuilder 集成

**问题**：QueryBuilder 需要知道数据库类型来生成正确的 SQL

**解决方案**：

- QueryBuilder 已经接受 `DbDriver` 参数
- 可以通过 `DB::get_driver()` 获取

### 5.5 类型约束复杂性

**问题**：泛型约束可能变得复杂

**解决方案**：

- 使用 trait alias（如果 Rust 版本支持）
- 或者定义辅助 trait 来简化约束

## 六、兼容性策略

### 6.1 向后兼容选项

**选项 A：完全替换（激进）**

- 移除所有数据库特定函数
- 强制用户使用泛型版本
- 优点：代码最简洁
- 缺点：破坏性变更

**选项 B：兼容层（保守）**

- 保留数据库特定函数作为兼容层
- 内部调用泛型版本
- 优点：向后兼容
- 缺点：代码冗余

**选项 C：渐进式迁移（推荐）**

- 阶段 2-4：保留兼容层
- 阶段 5：提供新的 trait API
- 阶段 6：标记旧 API 为 deprecated
- 未来版本：移除旧 API

### 6.2 推荐策略

采用 **选项 C（渐进式迁移）**：

1. 先实现泛型版本
2. 保留旧函数作为兼容层
3. 标记旧 API 为 `#[deprecated]`
4. 在下一个主版本中移除旧 API

## 七、测试策略

### 7.1 单元测试

- 为 `DatabaseInfo` trait 实现编写测试
- 为每个泛型函数编写测试
- 覆盖三个数据库的测试用例

### 7.2 集成测试

- 使用现有的 example 作为集成测试
- 确保所有功能正常工作

### 7.3 性能测试

- 对比新旧实现的性能
- 确保没有性能回退

## 八、文档更新

### 8.1 API 文档

- 更新所有函数文档
- 添加泛型参数说明
- 提供使用示例

### 8.2 迁移指南

- 编写从旧 API 迁移到新 API 的指南
- 提供代码示例

### 8.3 示例代码

- 更新所有 example
- 展示新的 API 使用方式

## 九、时间估算

| 阶段                       | 预估时间     | 备注             |
| -------------------------- | ------------ | ---------------- |
| 阶段 1：DatabaseInfo trait | 1-2 天       | 基础工作         |
| 阶段 2：find_by_id 试点    | 2-3 天       | 验证方案         |
| 阶段 3：迁移查询函数       | 3-5 天       | 5 个函数         |
| 阶段 4：迁移删除函数       | 1-2 天       | 3 个函数         |
| 阶段 5：更新 trait         | 3-5 天       | 最复杂           |
| 阶段 6：清理优化           | 2-3 天       | 收尾工作         |
| **总计**                   | **12-20 天** | 取决于测试和文档 |

## 十、风险控制

### 10.1 技术风险

- **风险**：类型约束过于复杂
- **缓解**：分阶段实施，及时调整

### 10.2 兼容性风险

- **风险**：破坏现有代码
- **缓解**：保留兼容层，渐进式迁移

### 10.3 性能风险

- **风险**：泛型可能影响性能
- **缓解**：性能测试，Rust 零成本抽象

## 十一、成功标准

1. ✅ 代码重复减少 70% 以上
2. ✅ 所有测试通过
3. ✅ 性能无回退
4. ✅ API 更简洁易用
5. ✅ 文档完整更新
6. ✅ 向后兼容（或提供清晰的迁移路径）

## 十二、下一步行动

1. **立即开始阶段 1**：定义 `DatabaseInfo` trait
2. **准备测试环境**：确保所有数据库测试可用
3. **代码审查准备**：准备每个阶段的代码审查点

---

**文档版本**：v1.0  
**创建日期**：2025-01-XX  
**最后更新**：2025-01-XX
