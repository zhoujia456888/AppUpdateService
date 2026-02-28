use crate::api::app_channel::app_channel_router;
use crate::api::ping::ping_router;
use crate::api::users::{auth_token, user_router_not_auth, users_router};
use crate::db::establish_connection_pool;
use crate::middleware::access_log::AccessLog;
use crate::model::jwt::{AccessTokenClaims, get_jwt_secret_key};
use crate::utils::json_error_catcher::json_error_catcher;
use moka::future::Cache;
use salvo::catcher::Catcher;
use salvo::jwt_auth::{ConstDecoder, HeaderFinder};
use salvo::prelude::*;
use salvo_oapi::SecurityScheme;
use salvo_oapi::security::{Http, HttpAuthScheme};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use crate::api::app_manage::app_manage_router;

pub async fn run() {
    //数据库
    let pool = Arc::new(establish_connection_pool());

    //登录验证码缓存
    let captcha_cache: Cache<String, String> = Cache::builder()
        .time_to_live(Duration::from_secs(60 * 10))
        .max_capacity(10_000)
        .build();

    let auth_handler: JwtAuth<AccessTokenClaims, _> =
        JwtAuth::new(ConstDecoder::from_secret(get_jwt_secret_key().as_bytes()))
            .finders(vec![Box::new(HeaderFinder::new())])
            .force_passed(true);

    //设置端口为5800
    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    //添加数据库配置
    let router = Router::new()
        .hoop(AccessLog {})
        .hoop(affix_state::inject(pool).inject(Arc::new(captcha_cache)));

    //添加不需要token的接口
    let router = router.push(
        Router::with_path("api")
            .push(ping_router())
            .push(user_router_not_auth()),
    );

    //添加需要token的接口路由配置
    let router = router.push(
        Router::with_path("api")
            .hoop(auth_handler)  // JwtAuth 中间件放在这里
            .hoop(auth_token)
            .push(users_router())
            .push(app_channel_router())
            .push(app_manage_router()),
    );

    //打印路径用于调试
    info!("{:?}", router);

    //设置api-doc
    let doc = OpenApi::new("AppUpdateService API", "0.0.1")
        .add_security_scheme(
            "Authorization",
            SecurityScheme::Http(
                Http::new(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description("JWT Bearer token. Example: '{token}'".to_string()),
            ),
        )
        .merge_router(&router);
    let router = router
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(SwaggerUi::new("/api-doc/openapi.json").into_router("/swagger-ui"));

    let service = Service::new(router);
    let catcher = Catcher::default().hoop(json_error_catcher);

    let service = service.catcher(catcher);
    //开始服务请求
    Server::new(acceptor).serve(service).await;
}
