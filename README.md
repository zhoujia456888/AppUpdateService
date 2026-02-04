# APP更新后台服务

app更新后台服务，用于APP更新。

## 主要功能

1.用户管理（JWT认证（access_token和refresh_token）、用户登陆）

2.APP渠道管理

3.APP上传下载

4.OpenAPI([swagger](http://localhost:5800/swagger-ui/#/))

5.数据库（[diesel](https://diesel.rs/),[PostgreSQL](https://www.postgresql.org/)）

## 致谢

[Salvo](https://github.com/salvo-rs/salvo) -- 风驰电掣的 Rust Web 框架

## [diesel](https://diesel.rs/) 命令

```
//创建数据库
diesel setup

//创建迁移
diesel migration generate update

//运行迁移
diesel migration run

//重置迁移
diesel migration redo

///完全重置
diesel database reset
```


