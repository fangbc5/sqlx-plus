CREATE TABLE "user2" (
    "id" BIGSERIAL,
    "tinyint_field" SMALLINT NOT NULL DEFAULT 1,
    "smallint_field" SMALLINT,
    "int_field" INTEGER NOT NULL DEFAULT 100,
    "bigint_field" BIGINT,
    "float_field" REAL NOT NULL DEFAULT 3.14,
    "double_field" DOUBLE PRECISION DEFAULT 2.71828,
    "bool_field_true" BOOLEAN NOT NULL DEFAULT TRUE,
    "bool_field_false" BOOLEAN NOT NULL DEFAULT FALSE,
    "bool_field_nullable" BOOLEAN DEFAULT TRUE,
    "varchar_not_null" VARCHAR(50) NOT NULL,
    "varchar_nullable" VARCHAR(100) DEFAULT '',
    "text_field" TEXT,
    "date_field" DATE,
    "datetime_field" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "timestamp_field" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "time_field" TIME WITH TIME ZONE,
    "json_field" JSONB,
    "blob_field" BYTEA,
    "unique_field" VARCHAR(50),
    "index_field" VARCHAR(50),
    "composite_field1" VARCHAR(50),
    "composite_field2" VARCHAR(50),
    PRIMARY KEY ("id"),
    CONSTRAINT "uk_user2_unique_field" UNIQUE ("unique_field")
);

CREATE INDEX "idx_user2_index_field" ON "user2" ("index_field");
CREATE INDEX "idx_composite" ON "user2" ("composite_field1", "composite_field2");


COMMENT ON COLUMN "user2"."id" IS '主键ID';
COMMENT ON COLUMN "user2"."tinyint_field" IS 'TINYINT类型';
COMMENT ON COLUMN "user2"."smallint_field" IS 'SMALLINT类型';
COMMENT ON COLUMN "user2"."int_field" IS 'INT类型';
COMMENT ON COLUMN "user2"."bigint_field" IS 'BIGINT类型';
COMMENT ON COLUMN "user2"."float_field" IS 'FLOAT类型';
COMMENT ON COLUMN "user2"."double_field" IS 'DOUBLE类型';
COMMENT ON COLUMN "user2"."bool_field_true" IS 'BOOL类型 - 默认true';
COMMENT ON COLUMN "user2"."bool_field_false" IS 'BOOL类型 - 默认false';
COMMENT ON COLUMN "user2"."bool_field_nullable" IS 'BOOL类型 - 可空';
COMMENT ON COLUMN "user2"."varchar_not_null" IS 'VARCHAR类型 - 非空';
COMMENT ON COLUMN "user2"."varchar_nullable" IS 'VARCHAR类型 - 可空';
COMMENT ON COLUMN "user2"."text_field" IS 'TEXT类型';
COMMENT ON COLUMN "user2"."date_field" IS 'DATE类型';
COMMENT ON COLUMN "user2"."datetime_field" IS 'DATETIME类型';
COMMENT ON COLUMN "user2"."timestamp_field" IS 'TIMESTAMP类型';
COMMENT ON COLUMN "user2"."time_field" IS 'TIME类型';
COMMENT ON COLUMN "user2"."json_field" IS 'JSON类型';
COMMENT ON COLUMN "user2"."blob_field" IS 'BLOB类型';
COMMENT ON COLUMN "user2"."unique_field" IS '唯一索引字段';
COMMENT ON COLUMN "user2"."index_field" IS '普通索引字段';
COMMENT ON COLUMN "user2"."composite_field1" IS '联合索引字段1';
COMMENT ON COLUMN "user2"."composite_field2" IS '联合索引字段2';


COMMENT ON TABLE "user2" IS '用户表2 - 测试各种数据类型';
