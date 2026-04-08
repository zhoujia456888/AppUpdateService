ALTER TABLE "app_manage"
ALTER COLUMN "version_code" TYPE INT
USING "version_code"::INT;
