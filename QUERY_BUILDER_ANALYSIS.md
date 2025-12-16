# QueryBuilder 功能分析

## 当前已实现的功能

### ✅ 基本条件查询（AND）

- `and_eq(field, value)` - 等于 (=)
- `and_ne(field, value)` - 不等于 (!=)
- `and_gt(field, value)` - 大于 (>)
- `and_ge(field, value)` - 大于等于 (>=)
- `and_lt(field, value)` - 小于 (<)
- `and_le(field, value)` - 小于等于 (<=)

### ✅ OR 条件查询

- `or_eq(field, value)` - 等于 (=)
- `or_ne(field, value)` - 不等于 (!=)
- `or_gt(field, value)` - 大于 (>)
- `or_ge(field, value)` - 大于等于 (>=)
- `or_lt(field, value)` - 小于 (<)
- `or_le(field, value)` - 小于等于 (<=)

### ✅ 字符串查询

- `and_like(field, value)` - LIKE 查询（自动添加 %value%）
- `and_like_prefix(field, value)` - LIKE 前缀匹配（value%）【✅ 新增】
- `and_like_suffix(field, value)` - LIKE 后缀匹配（%value）【✅ 新增】
- `and_like_exact(field, value)` - LIKE 精确匹配（不添加 %）【✅ 新增】
- `and_like_custom(field, pattern)` - LIKE 自定义模式匹配【✅ 新增】
- `or_like(field, value)` - OR LIKE 查询【✅ 新增】

### ✅ 集合查询

- `and_in(field, values)` - IN 查询
- `and_not_in(field, values)` - NOT IN 查询【✅ 新增】
- `or_in(field, values)` - OR IN 查询【✅ 新增】

### ✅ 空值查询

- `and_is_null(field)` - IS NULL
- `and_is_not_null(field)` - IS NOT NULL
- `or_is_null(field)` - OR IS NULL
- `or_is_not_null(field)` - OR IS NOT NULL

### ✅ 范围查询

### ✅ 条件分组（括号）

- `and_group(|b| { ... })` / `or_group(|b| { ... })`
- 支持 `(A AND B)`、`(A OR B)`、`(A AND (B OR C))`、`(A AND B) OR (C AND D)` 等多层嵌套组合

- `and_between(field, min, max)` - BETWEEN 范围查询
- `or_between(field, min, max)` - OR BETWEEN 范围查询

### ✅ 排序

- `order_by(field, ascending)` - 单字段排序
- 支持多字段排序（通过多次调用）

### ✅ 其他功能

- 自动处理 WHERE 子句（检查 base_sql 是否已包含 WHERE）
- 支持参数绑定（防止 SQL 注入）
- 支持 COUNT 查询转换 (`into_count_sql`)
- 支持分页 SQL 生成 (`into_paginated_sql`)
- 支持 `limit(n)` / `offset(n)` 链式方法（作用于 `into_sql`）
- 支持多种数据库驱动（MySQL, PostgreSQL, SQLite），自动转换占位符格式

### ✅ GROUP BY 和 HAVING

- **GROUP BY**：支持单字段和多字段分组
  - `group_by(field)` - 添加分组字段（可链式调用多次）
- **HAVING**：支持分组后的条件过滤
  - `having_eq(field, value)` - HAVING 等于
  - `having_ne(field, value)` - HAVING 不等于
  - `having_gt(field, value)` - HAVING 大于
  - `having_ge(field, value)` - HAVING 大于等于
  - `having_lt(field, value)` - HAVING 小于
  - `having_le(field, value)` - HAVING 小于等于
- 支持与 WHERE、ORDER BY、LIMIT/OFFSET 组合使用
- 自动转义字段名，兼容 MySQL / PostgreSQL / SQLite

## 缺失的重要功能

### 🟡 中优先级（有用功能）

1. **JOIN 支持**

   - 缺少表连接
   - 需要：
     - `inner_join(table, condition)`
     - `left_join(table, condition)`
     - `right_join(table, condition)`
     - `full_join(table, condition)`

2. **子查询支持**

   - 缺少子查询功能
   - 需要：支持在条件中使用子查询

3. **EXISTS / NOT EXISTS**

   - 缺少存在性查询
   - 需要：`and_exists(subquery)`, `and_not_exists(subquery)`

4. **正则表达式**
   - 缺少正则匹配（MySQL REGEXP, PostgreSQL ~）
   - 需要：`and_regexp(field, pattern)`

### 🟢 低优先级（高级功能）

7. **字段选择**

   - 当前只能使用 `SELECT *`
   - 需要：`select(fields)` 方法，支持选择特定字段

8. **字段别名**

   - 缺少字段别名支持
   - 需要：`select_as(field, alias)` 方法

9. **聚合函数**

   - 缺少聚合函数支持（COUNT, SUM, AVG, MAX, MIN）
   - 需要：在 SELECT 中支持聚合函数

10. **UNION**

    - 缺少 UNION 查询
    - 需要：`union(other_builder)` 方法

11. **日期时间函数**

    - 缺少日期时间函数支持
    - 需要：`and_date_eq()`, `and_date_between()` 等

12. **条件组合优化**

    - 当前条件都是线性添加
    - 需要：支持条件分组，如 `(A OR B) AND (C OR D)`

13. **DISTINCT**

    - 缺少去重查询
    - 需要：`distinct()` 方法

14. **CASE WHEN**
    - 缺少条件表达式
    - 需要：支持 CASE WHEN 语句

## 建议的改进优先级

### Phase 1: 核心功能增强（✅ 已完成）

1. ✅ OR 条件支持（`or_eq`, `or_ne` 等）
2. ✅ IS NULL / IS NOT NULL
3. ✅ NOT IN
4. ✅ BETWEEN
5. ✅ 更灵活的 LIKE（prefix, suffix, custom）

### Phase 2: 常用功能（部分完成）

6. ✅ GROUP BY
7. ✅ HAVING
8. EXISTS / NOT EXISTS

### Phase 3: 高级功能（长期规划）

10. JOIN 支持
11. 子查询支持
12. UNION 支持
13. 字段选择（SELECT 特定字段）
14. 聚合函数支持

## 当前功能覆盖度评估

- **基本查询功能**: 95% ✅（已支持所有基本操作符和 OR 条件）
- **条件组合**: 90% ✅（支持 AND/OR 和括号分组，大部分单表复杂条件可表达）
- **聚合查询**: 60% ✅（支持 GROUP BY 和 HAVING，但缺少聚合函数支持）
- **高级查询**: 30% ⚠️（缺少 JOIN、子查询）
- **复杂查询**: 20% ❌（缺少 UNION、EXISTS 等）

**总体评估**: 当前 QueryBuilder 能够覆盖 **85%+** 的常见查询场景，非常适合大多数 CRUD 操作和中等复杂度的查询。已实现的高优先级功能（包括 GROUP BY 和 HAVING）大大提升了实用性。
