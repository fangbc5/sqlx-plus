CREATE TABLE `user2` (
    `id` BIGINT AUTO_INCREMENT COMMENT '主键ID',
    `tinyint_field` TINYINT NOT NULL DEFAULT 1 COMMENT 'TINYINT类型',
    `smallint_field` TINYINT COMMENT 'SMALLINT类型',
    `int_field` INT NOT NULL DEFAULT 100 COMMENT 'INT类型',
    `bigint_field` BIGINT COMMENT 'BIGINT类型',
    `float_field` FLOAT NOT NULL DEFAULT 3.14 COMMENT 'FLOAT类型',
    `double_field` DOUBLE DEFAULT 2.71828 COMMENT 'DOUBLE类型',
    `bool_field_true` TINYINT(1) NOT NULL DEFAULT 1 COMMENT 'BOOL类型 - 默认true',
    `bool_field_false` TINYINT(1) NOT NULL DEFAULT 0 COMMENT 'BOOL类型 - 默认false',
    `bool_field_nullable` TINYINT(1) DEFAULT 1 COMMENT 'BOOL类型 - 可空',
    `varchar_not_null` VARCHAR(50) NOT NULL COMMENT 'VARCHAR类型 - 非空',
    `varchar_nullable` VARCHAR(100) DEFAULT '' COMMENT 'VARCHAR类型 - 可空',
    `text_field` TEXT COMMENT 'TEXT类型',
    `date_field` DATE COMMENT 'DATE类型',
    `datetime_field` TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'DATETIME类型',
    `timestamp_field` TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'TIMESTAMP类型',
    `time_field` TIME(3) COMMENT 'TIME类型',
    `json_field` JSON COMMENT 'JSON类型',
    `blob_field` BLOB COMMENT 'BLOB类型',
    `unique_field` VARCHAR(50) COMMENT '唯一索引字段',
    `index_field` VARCHAR(50) COMMENT '普通索引字段',
    `composite_field1` VARCHAR(50) COMMENT '联合索引字段1',
    `composite_field2` VARCHAR(50) COMMENT '联合索引字段2',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_user2_unique_field` (`unique_field`)
) COMMENT '用户表2 - 测试各种数据类型';

CREATE INDEX `idx_user2_index_field` ON `user2` (`index_field`);
CREATE INDEX `idx_composite` ON `user2` (`composite_field1`, `composite_field2`);
