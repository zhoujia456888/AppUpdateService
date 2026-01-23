-- Your SQL goes here
CREATE TABLE "users"
(
    "id"          UUID    NOT NULL PRIMARY KEY,
    "username"    VARCHAR NOT NULL,
    "password"    VARCHAR NOT NULL,
    "full_name"   VARCHAR NOT NULL,
    "token"       VARCHAR NOT NULL,
    "create_time" VARCHAR NOT NULL,
);