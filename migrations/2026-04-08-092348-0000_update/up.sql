ALTER TABLE "app_manage"
ALTER COLUMN "version_code" TYPE VARCHAR
USING "version_code"::VARCHAR;
