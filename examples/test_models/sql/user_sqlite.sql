CREATE TABLE "user" (
    "id" INTEGER -- 主键ID,
    "system_type" INTEGER NOT NULL DEFAULT 1 -- 系统类型,
    "user_type" INTEGER DEFAULT 3 -- 用户类型,
    "username" TEXT -- 用户名,
    "nick_name" TEXT -- 昵称,
    "real_name" TEXT -- 真实姓名,
    "avatar" TEXT NOT NULL DEFAULT '' -- 头像URL,
    "avatar_update_time" TEXT -- 头像更新时间,
    "email" TEXT -- 邮箱地址,
    "region" TEXT -- 地区代码,
    "mobile" TEXT -- 手机号,
    "id_card" TEXT -- 身份证号,
    "wx_open_id" TEXT -- 微信OpenID,
    "dd_open_id" TEXT -- 钉钉OpenID,
    "sex" INTEGER DEFAULT 0 -- 性别：0-未知，1-男，2-女,
    "state" INTEGER DEFAULT 1 -- 状态：1-正常,
    "user_state_id" INTEGER -- 用户状态ID,
    "resume" TEXT -- 简历,
    "work_describe" TEXT -- 工作描述,
    "item_id" INTEGER -- 项目ID,
    "context" INTEGER DEFAULT 0 -- 上下文,
    "num" INTEGER DEFAULT 10 -- 数量,
    "password" TEXT NOT NULL -- 密码,
    "salt" TEXT -- 盐值,
    "password_error_num" INTEGER DEFAULT 0 -- 密码错误次数,
    "password_error_last_time" TEXT -- 密码错误最后时间,
    "password_expire_time" TEXT -- 密码过期时间,
    "last_opt_time" TEXT DEFAULT CURRENT_TIMESTAMP(3) -- 最后操作时间,
    "last_login_time" TEXT -- 最后登录时间,
    "ip_info" TEXT -- IP信息,
    "create_time" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP(3) -- 创建时间,
    "update_time" TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP(3) -- 更新时间,
    "create_by" INTEGER DEFAULT 1 -- 创建人ID,
    "update_by" INTEGER -- 更新人ID,
    "is_del" INTEGER NOT NULL DEFAULT 0 -- 是否删除：0-未删除，1-已删除,
    "readonly" INTEGER DEFAULT 0 -- 是否只读：0-否，1-是,
    PRIMARY KEY ("id"),
    UNIQUE ("email")
);

CREATE INDEX "idx_user_username" ON "user" ("username");
CREATE INDEX "idx_user_is_del" ON "user" ("is_del");


-- 表注释: 用户表
