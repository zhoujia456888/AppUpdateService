-- Your SQL goes here
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

CREATE Table "app_channel"
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
    "app_version"      VARCHAR   NOT NULL,
    "app_download_url" VARCHAR   NOT NULL,
    "create_user_id"   UUID      NOT NULL,
    "channel_id"       UUID      NOT NULL,
    "create_time"      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "update_time"      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "is_delete"        BOOLEAN   NOT NULL DEFAULT FALSE,
    CONSTRAINT fk_app_manage_users FOREIGN KEY (create_user_id) REFERENCES users (id),
    CONSTRAINT fk_app_manage_channel FOREIGN KEY (channel_id) REFERENCES app_channel (id)
);