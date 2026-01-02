CREATE TABLE "user2" (
    "id" BIGSERIAL,
    "system_type" SMALLINT NOT NULL DEFAULT 1,
    "is_del" SMALLINT NOT NULL DEFAULT 0,
    "username" VARCHAR(255),
    "email" VARCHAR(255),
    "status" SMALLINT DEFAULT 1,
    "create_time" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    "update_time" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY ("id"),
    UNIQUE KEY "uk_user2_username" ("username"),
    UNIQUE KEY "idx_email" ("email")
);

CREATE INDEX "idx_user2_system_type" ON "user2" ("system_type");
CREATE INDEX "idx_email_status" ON "user2" ("email", "status");
