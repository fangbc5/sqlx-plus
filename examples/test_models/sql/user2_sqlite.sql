CREATE TABLE "user2" (
    "id" INTEGER -- 主键ID,
    "tinyint_field" INTEGER NOT NULL DEFAULT 1 -- TINYINT类型,
    "smallint_field" INTEGER -- SMALLINT类型,
    "int_field" INTEGER NOT NULL DEFAULT 100 -- INT类型,
    "bigint_field" INTEGER -- BIGINT类型,
    "float_field" REAL NOT NULL DEFAULT 3.14 -- FLOAT类型,
    "double_field" REAL DEFAULT 2.71828 -- DOUBLE类型,
    "bool_field_true" INTEGER NOT NULL DEFAULT 1 -- BOOL类型 - 默认true,
    "bool_field_false" INTEGER NOT NULL DEFAULT 0 -- BOOL类型 - 默认false,
    "bool_field_nullable" INTEGER DEFAULT 1 -- BOOL类型 - 可空,
    "varchar_not_null" TEXT NOT NULL -- VARCHAR类型 - 非空,
    "varchar_nullable" TEXT DEFAULT '' -- VARCHAR类型 - 可空,
    "text_field" TEXT -- TEXT类型,
    "date_field" TEXT -- DATE类型,
    "datetime_field" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP -- DATETIME类型,
    "timestamp_field" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP -- TIMESTAMP类型,
    "time_field" TEXT -- TIME类型,
    "json_field" TEXT -- JSON类型,
    "blob_field" BLOB -- BLOB类型,
    "unique_field" TEXT -- 唯一索引字段,
    "index_field" TEXT -- 普通索引字段,
    "composite_field1" TEXT -- 联合索引字段1,
    "composite_field2" TEXT -- 联合索引字段2,
    PRIMARY KEY ("id"),
    UNIQUE ("unique_field")
);

CREATE INDEX "idx_user2_index_field" ON "user2" ("index_field");
CREATE INDEX "idx_composite" ON "user2" ("composite_field1", "composite_field2");


-- 表注释: 用户表2 - 测试各种数据类型
