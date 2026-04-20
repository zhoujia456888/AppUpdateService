use crate::api::app_channel::app_channel_router;
use crate::api::app_manage::{app_check_update, app_manage_router, get_app_info};
use crate::api::operation_log::operation_log_router;
use crate::api::ping::ping_router;
use crate::api::users::{auth_token, user_router_not_auth, users_router};
use crate::db::establish_connection_pool;
use crate::middleware::access_log::AccessLog;
use crate::model::jwt::{AccessTokenClaims, get_jwt_secret_key};
use crate::utils::app_manage_cleanup_task::start_app_manage_cleanup_task;
use crate::utils::json_error_catcher::json_error_catcher;
use moka::future::Cache;
use salvo::catcher::Catcher;
use salvo::fs::NamedFile;
use salvo::jwt_auth::{ConstDecoder, HeaderFinder};
use salvo::prelude::*;
use salvo_oapi::SecurityScheme;
use salvo_oapi::security::{Http, HttpAuthScheme};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

const PUBLIC_APP_MANAGE_DIR: &str = "app_manage";

fn build_public_router() -> Router {
    Router::with_path("api").push(
        Router::with_path("public")
            .push(
                Router::with_path("app_manage")
                    .push(Router::with_path("icon").get(public_app_manage_icon_file))
                    .push(Router::with_path("apk").get(public_app_manage_apk_file))
                    .push(Router::with_path("app_check_update").post(app_check_update))
                    .push(Router::with_path("get_app_info").post(get_app_info)),
            )
            .push(ping_router())
            .push(user_router_not_auth()),
    )
}

fn build_protected_router(auth_handler: JwtAuth<AccessTokenClaims, ConstDecoder>) -> Router {
    Router::with_path("api")
        .hoop(auth_handler)
        .hoop(auth_token)
        .push(users_router())
        .push(operation_log_router())
        .push(app_channel_router())
        .push(app_manage_router())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_router_should_expose_get_app_info() {
        let debug = format!("{:?}", build_public_router());
        assert!(debug.contains("get_app_info"));
    }
}

pub async fn run() {
    //数据库
    let pool = Arc::new(establish_connection_pool());
    start_app_manage_cleanup_task(pool.clone());

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

    // 添加公开接口：不需要 Token，可直接访问
    let public_router = build_public_router();

    // 添加受保护接口：统一要求 Token 鉴权
    let protected_router = build_protected_router(auth_handler);

    let router = router.push(public_router).push(protected_router);

    info!("{:?}", router);

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
    Server::new(acceptor).serve(service).await;
}

// 公开应用图标文件
#[handler]
async fn public_app_manage_icon_file(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    serve_public_app_manage_file("icons", req, depot, res).await;
}

// 公开应用APK文件
#[handler]
async fn public_app_manage_apk_file(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    serve_public_app_manage_file("apk", req, depot, res).await;
}

// 公开应用管理文件
pub async fn serve_public_app_manage_file(
    sub_dir: &str,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    let Some(filename) = req
        .query::<String>("name")
        .map(|value| value.replace('\\', "/"))
        .map(|value| value.trim_start_matches('/').to_string())
        .filter(|value| !value.trim().is_empty())
    else {
        res.status_code(StatusCode::BAD_REQUEST);
        return;
    };

    let relative_path = Path::new(&filename);
    if relative_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        res.status_code(StatusCode::FORBIDDEN);
        return;
    }

    let full_path = PathBuf::from(PUBLIC_APP_MANAGE_DIR)
        .join(sub_dir)
        .join(relative_path);
    match NamedFile::builder(full_path).build().await {
        Ok(file) => {
            file.write(req, depot, res).await;
        }
        Err(_) => {
            res.status_code(StatusCode::NOT_FOUND);
        }
    }
}
