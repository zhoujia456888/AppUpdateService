CREATE TABLE "users"
(
    "id"            UUID      NOT NULL PRIMARY KEY,
    "username"      VARCHAR   NOT NULL,
    "password"      VARCHAR   NOT NULL,
    "full_name"     VARCHAR   NOT NULL,
    "access_token"  VARCHAR   NOT NULL,
    "refresh_token" VARCHAR   NOT NULL,
    "create_time"   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "update_time"   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "is_delete"     BOOLEAN   NOT NULL DEFAULT FALSE
);

CREATE TABLE "app_channel"
(
    "id"             UUID      NOT NULL PRIMARY KEY,
    "channel_name"   VARCHAR   NOT NULL,
    "create_user_id" UUID      NOT NULL,
    "create_time"    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "update_time"    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "is_delete"      BOOLEAN   NOT NULL DEFAULT FALSE,
    CONSTRAINT fk_app_channel_users FOREIGN KEY (create_user_id) REFERENCES users (id)
);

CREATE TABLE "app_manage"
(
    "id"               UUID      NOT NULL PRIMARY KEY,
    "app_name"         VARCHAR   NOT NULL,
    "app_download_url" VARCHAR   NOT NULL,
    "create_user_id"   UUID      NOT NULL,
    "channel_id"       UUID      NOT NULL,
    "create_time"      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "update_time"      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "is_delete"        BOOLEAN   NOT NULL DEFAULT FALSE,
    "file_path"        VARCHAR,
    "file_name"        VARCHAR,
    "package_name"     VARCHAR,
    "app_icon_path"    VARCHAR,
    "version_name"     VARCHAR,
    "version_code"     VARCHAR   NOT NULL DEFAULT '',
    "file_size"        BIGINT    NOT NULL DEFAULT 0,
    "channel_name"     VARCHAR,
    "update_log"       VARCHAR,
    CONSTRAINT fk_app_manage_users FOREIGN KEY (create_user_id) REFERENCES users (id),
    CONSTRAINT fk_app_manage_channel FOREIGN KEY (channel_id) REFERENCES app_channel (id)
);

CREATE TABLE "operation_log"
(
    "id"               UUID      NOT NULL PRIMARY KEY,
    "user_id"          UUID      NOT NULL,
    "username"         VARCHAR   NOT NULL,
    "operation_type"   VARCHAR   NOT NULL,
    "operation_detail" VARCHAR   NOT NULL,
    "create_time"      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_operation_log_users FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE TABLE "auth_captcha"
(
    "captcha_id"   VARCHAR   NOT NULL PRIMARY KEY,
    "captcha_text" VARCHAR   NOT NULL,
    "create_time"  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "expires_at"   TIMESTAMP NOT NULL
);

CREATE INDEX "idx_auth_captcha_expires_at" ON "auth_captcha" ("expires_at");
