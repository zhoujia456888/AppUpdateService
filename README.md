# AppUpdateService

AppUpdateService 是一个基于 Rust 开发的 APP 更新后台服务，主要面向 Android APK 的上传、版本管理、渠道分发和更新检查场景。

它不仅提供用户登录鉴权、渠道管理、应用版本管理等后台能力，还负责 APK 文件与应用图标的本地存储及公开访问，适合用作企业内部 APP 分发平台、测试包管理后台或私有化更新服务。

## 项目定位

这是一个面向 **Android APP 更新与分发** 的后端服务，核心目标包括：

- 管理后台用户及登录态
- 管理 APP 发布渠道
- 上传 APK 并自动解析元数据
- 发布应用版本并记录更新日志
- 为客户端提供检查更新接口
- 提供 APK 和图标的公开访问地址
- 通过 OpenAPI / Swagger 提供接口文档

## 技术栈

- **语言**：Rust
- **Web 框架**： [Salvo](https://github.com/salvo-rs/salvo)
- **异步运行时**：Tokio
- **数据库 ORM**： [Diesel](https://diesel.rs/)
- **数据库**： [PostgreSQL](https://www.postgresql.org/)
- **认证方式**：JWT（access token + refresh token）
- **密码安全**：Argon2
- **验证码缓存**：Moka
- **API 文档**：OpenAPI / Swagger UI
- **APK 元数据解析**：apk-info
- **容器化部署**：Docker

## 核心功能

### 1. 用户管理与认证

支持完整的基础账号体系：

- 获取登录/注册验证码
- 用户注册
- 用户登录
- 刷新 Token
- 获取当前用户信息

认证方式采用 JWT，并结合数据库中的 access_token / refresh_token 进行校验。

### 2. APP 渠道管理

支持按渠道管理应用版本，适合多环境、多渠道发布场景，例如：

- official
- test
- gray
- partner

支持的能力包括：

- 创建渠道
- 分页查询渠道
- 搜索渠道
- 更新渠道
- 软删除渠道
- 完全删除渠道

### 3. APK 上传与元数据提取

上传 APK 后，系统会自动完成以下操作：

- 保存 APK 文件到本地目录
- 解析 APK 包名
- 提取应用名称
- 提取版本号与版本编码
- 提取应用图标
- 计算文件大小

上传后的文件默认存放在：

- `app_manage/apk/`：APK 文件
- `app_manage/icons/`：应用图标

### 4. 应用版本发布

APK 上传成功后，可以继续完成版本发布，将以下信息写入数据库：

- 应用名称
- 包名
- 渠道信息
- 文件路径
- 图标路径
- 版本号 / 版本编码
- 文件大小
- 更新日志

### 5. 客户端检查更新

服务提供公开接口，客户端可以通过以下条件检查最新版本：

- `package_name`
- `channel_name`

返回内容包括：

- 应用名称
- 包名
- 渠道名
- 最新版本号
- 最新版本编码
- 下载地址

### 6. 公开接口与鉴权接口划分

项目中的接口按访问方式分为两类：

#### 公开接口

统一挂载在：

- `/api/public/...`

适用于客户端直接访问、下载资源或无需登录即可调用的接口，例如：

- 应用图标访问：`/api/public/app_manage/icon?name=xxx.png`
- APK 下载：`/api/public/app_manage/apk?name=xxx.apk`
- 检查更新：`/api/public/app_manage/app_check_update`
- 获取应用详情：`/api/public/app_manage/get_app_info`
- 登录/注册/验证码/刷新 Token 等无需登录的用户接口

#### 需要 Token 鉴权的接口

统一挂载在：

- `/api/...`

并由统一的 JWT 鉴权中间件保护，适用于后台管理操作，例如：

- 用户信息接口
- APP 渠道管理接口
- APP 上传与发布接口
- 应用列表查询等后台管理接口

这种划分的目的是：

- 明确哪些接口可以匿名访问
- 明确哪些接口必须经过 Token 校验
- 避免公开资源接口和后台管理接口混在一起
- 让客户端、前端和后端都更容易理解路由语义

## 项目结构

```text
AppUpdateService/
├── app_manage/              # 本地存储的 APK 和图标资源
├── logs/                    # 日志目录
├── migrations/              # Diesel 数据库迁移文件
├── src/
│   ├── api/                 # API 路由与业务接口
│   ├── logging/             # 日志初始化
│   ├── middleware/          # 中间件
│   ├── model/               # 请求/响应/实体模型
│   ├── utils/               # 工具模块（JWT、密码、APK 解析等）
│   ├── db.rs                # 数据库连接池
│   ├── main.rs              # 程序入口
│   ├── schema.rs            # Diesel 自动生成的表结构
│   └── server.rs            # 服务启动与路由注册
├── vendor/                  # 本地补丁依赖
├── Cargo.toml
├── Dockerfile
└── README.md
```

## 数据模型

项目当前主要包含以下 3 张核心表：

### `users`

用于存储用户信息与登录态：

- 用户名
- 密码哈希
- access_token
- refresh_token
- 创建时间 / 更新时间
- 删除标记

### `app_channel`

用于存储应用发布渠道：

- 渠道名称
- 创建人
- 创建时间 / 更新时间
- 删除标记

### `app_manage`

用于存储应用版本信息：

- 应用名称
- 下载地址
- 文件路径
- 文件名
- 包名
- 图标路径
- 版本名称
- 版本编码
- 文件大小
- 渠道 ID / 渠道名称
- 更新日志
- 创建人
- 创建时间 / 更新时间
- 删除标记

## 运行要求

启动前需要准备以下环境变量：

- `DATABASE_URL`
- `JWT_SECRET_KEY`
- `JWT_REFRESH_SECRET_KEY`
- `RUST_LOG`

默认服务监听端口：

- `5800`

## API 文档

项目启动后可访问 Swagger UI：

- <http://localhost:5800/swagger-ui>

OpenAPI JSON 地址：

- <http://localhost:5800/api-doc/openapi.json>

## Docker 部署

项目提供 `Dockerfile`，支持容器化部署。

容器默认：

- 暴露端口 `5800`
- 挂载日志目录 `/app/logs`
- 挂载资源目录 `/app/app_manage`

适合在单机或私有化环境中部署。

## 使用场景

本项目适合以下场景：

- 企业内部 APP 安装包分发
- 测试渠道 APK 管理
- 私有化 Android 更新服务
- 多渠道版本发布后台

## 致谢

- [Salvo](https://github.com/salvo-rs/salvo) — 风驰电掣的 Rust Web 框架
- [Diesel](https://diesel.rs/) — Rust 生态常用 ORM
- [PostgreSQL](https://www.postgresql.org/) — 稳定可靠的关系型数据库

## [diesel](https://diesel.rs/) 命令

```bash
# 安装 diesel_cli（PostgreSQL）
cargo install diesel_cli --no-default-features --features "postgres"

# 创建数据库
diesel setup

# 创建迁移
diesel migration generate update

# 运行迁移
diesel migration run

# 重置迁移
diesel migration redo

# 完全重置数据库
diesel database reset
```
