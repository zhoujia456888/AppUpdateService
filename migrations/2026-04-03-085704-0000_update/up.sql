-- Your SQL goes here
ALTER TABLE "app_manage"
ADD COLUMN "file_path" VARCHAR,
ADD COLUMN "file_name" VARCHAR,
ADD COLUMN "package_name" VARCHAR,
ADD COLUMN "app_icon_path" VARCHAR,
ADD COLUMN "version_name" VARCHAR,
ADD COLUMN "version_code" INT NOT NULL DEFAULT 0,
ADD COLUMN "file_size" BIGINT NOT NULL DEFAULT 0,
ADD COLUMN "channel_name" VARCHAR,
ADD COLUMN "update_log" VARCHAR;
