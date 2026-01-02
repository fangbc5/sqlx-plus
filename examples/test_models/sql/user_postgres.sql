CREATE TABLE "user" (
    "id" BIGSERIAL,
    "system_type" SMALLINT NOT NULL DEFAULT 1,
    "user_type" SMALLINT DEFAULT 3,
    "username" VARCHAR(255),
    "nick_name" VARCHAR(255),
    "real_name" VARCHAR(255),
    "avatar" VARCHAR(255) NOT NULL DEFAULT '',
    "avatar_update_time" TIMESTAMP WITH TIME ZONE,
    "email" VARCHAR(255),
    "region" VARCHAR(5),
    "mobile" VARCHAR(11),
    "id_card" VARCHAR(18),
    "wx_open_id" VARCHAR(255),
    "dd_open_id" VARCHAR(255),
    "sex" SMALLINT DEFAULT 0,
    "state" SMALLINT DEFAULT 1,
    "user_state_id" BIGINT,
    "resume" VARCHAR(200),
    "work_describe" VARCHAR(255),
    "item_id" BIGINT,
    "context" SMALLINT DEFAULT 0,
    "num" BIGINT DEFAULT 10,
    "password" VARCHAR(64) NOT NULL,
    "salt" VARCHAR(20),
    "password_error_num" INTEGER DEFAULT 0,
    "password_error_last_time" TIMESTAMP WITH TIME ZONE,
    "password_expire_time" TIMESTAMP WITH TIME ZONE,
    "last_opt_time" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP(3),
    "last_login_time" TIMESTAMP WITH TIME ZONE,
    "ip_info" JSONB,
    "create_time" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    "update_time" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    "create_by" BIGINT DEFAULT 1,
    "update_by" BIGINT,
    "is_del" SMALLINT NOT NULL DEFAULT 0,
    "readonly" SMALLINT DEFAULT 0,
    PRIMARY KEY ("id"),
    CONSTRAINT "uk_user_email" UNIQUE ("email")
);

CREATE INDEX "idx_user_username" ON "user" ("username");
CREATE INDEX "idx_user_is_del" ON "user" ("is_del");


COMMENT ON COLUMN "user"."id" IS '主键ID';
COMMENT ON COLUMN "user"."system_type" IS '系统类型';
COMMENT ON COLUMN "user"."user_type" IS '用户类型';
COMMENT ON COLUMN "user"."username" IS '用户名';
COMMENT ON COLUMN "user"."nick_name" IS '昵称';
COMMENT ON COLUMN "user"."real_name" IS '真实姓名';
COMMENT ON COLUMN "user"."avatar" IS '头像URL';
COMMENT ON COLUMN "user"."avatar_update_time" IS '头像更新时间';
COMMENT ON COLUMN "user"."email" IS '邮箱地址';
COMMENT ON COLUMN "user"."region" IS '地区代码';
COMMENT ON COLUMN "user"."mobile" IS '手机号';
COMMENT ON COLUMN "user"."id_card" IS '身份证号';
COMMENT ON COLUMN "user"."wx_open_id" IS '微信OpenID';
COMMENT ON COLUMN "user"."dd_open_id" IS '钉钉OpenID';
COMMENT ON COLUMN "user"."sex" IS '性别：0-未知，1-男，2-女';
COMMENT ON COLUMN "user"."state" IS '状态：1-正常';
COMMENT ON COLUMN "user"."user_state_id" IS '用户状态ID';
COMMENT ON COLUMN "user"."resume" IS '简历';
COMMENT ON COLUMN "user"."work_describe" IS '工作描述';
COMMENT ON COLUMN "user"."item_id" IS '项目ID';
COMMENT ON COLUMN "user"."context" IS '上下文';
COMMENT ON COLUMN "user"."num" IS '数量';
COMMENT ON COLUMN "user"."password" IS '密码';
COMMENT ON COLUMN "user"."salt" IS '盐值';
COMMENT ON COLUMN "user"."password_error_num" IS '密码错误次数';
COMMENT ON COLUMN "user"."password_error_last_time" IS '密码错误最后时间';
COMMENT ON COLUMN "user"."password_expire_time" IS '密码过期时间';
COMMENT ON COLUMN "user"."last_opt_time" IS '最后操作时间';
COMMENT ON COLUMN "user"."last_login_time" IS '最后登录时间';
COMMENT ON COLUMN "user"."ip_info" IS 'IP信息';
COMMENT ON COLUMN "user"."create_time" IS '创建时间';
COMMENT ON COLUMN "user"."update_time" IS '更新时间';
COMMENT ON COLUMN "user"."create_by" IS '创建人ID';
COMMENT ON COLUMN "user"."update_by" IS '更新人ID';
COMMENT ON COLUMN "user"."is_del" IS '是否删除：0-未删除，1-已删除';
COMMENT ON COLUMN "user"."readonly" IS '是否只读：0-否，1-是';


COMMENT ON TABLE "user" IS '用户表';
