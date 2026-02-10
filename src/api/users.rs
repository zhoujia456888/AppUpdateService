use crate::model::body::parse_json_body;
use crate::model::error::{ApiOut, AppError};
use crate::model::jwt::{AccessTokenClaims, RefreshTokenReq, TokenResp};
use crate::model::response::ApiResponse;
use crate::model::users::{
    CaptchaResp, LoginReq, LoginResp, RegisterReq, RegisterResp, User, UserInfoResp,
};
use crate::schema::*;
use crate::utils::auth_captcha_utils;
use crate::utils::auth_captcha_utils::get_captcha_cache;
use crate::utils::database_utils::connect_database;
use crate::utils::jwt_service::{
    generate_access_token, generate_refresh_token, refresh_access_token,
};
use crate::utils::password_utils::{hash_password, verify_password};
use chrono::Local;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use log::info;
use moka::future::Cache;
use salvo::prelude::*;
use salvo_oapi::endpoint;
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

#[endpoint(
    tags("Users"),
    summary = "获取登录注册验证码",
    description = "获取登录注册验证码"
)]
pub async fn get_auth_captcha(depot: &mut Depot) -> ApiOut<CaptchaResp> {
    let captcha = auth_captcha_utils::get_auth_captcha();

    let cache = match get_captcha_cache(depot) {
        Ok(value) => value,
        Err(value) => return value,
    };

    cache.insert(captcha.id.clone(), captcha.text.clone()).await;

    ApiOut::ok(CaptchaResp {
        captcha_id: captcha.id,
        captcha_img: captcha.img,
    })
}

#[endpoint(tags("Users"), summary = "用户注册", description = "用户注册",request_body = RegisterReq)]
pub async fn register(depot: &mut Depot, req: &mut Request) -> ApiOut<RegisterResp> {
    let register_req = match parse_json_body::<RegisterReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    if register_req.username.is_empty() {
        return ApiOut::err(AppError::BadRequest("用户名称不能为空".to_string()));
    }

    if register_req.password.is_empty() {
        return ApiOut::err(AppError::BadRequest("用户密码不能为空".to_string()));
    }

    //验证密码是否一致
    if register_req.password != register_req.confirm_password {
        return ApiOut::err(AppError::BadRequest("两次输入的密码不一致".to_string()));
    }

    //验证验证码
    if let Err(e) =
        validate_captcha(depot, &register_req.captcha_id, &register_req.captcha_code).await
    {
        return ApiOut::err(e);
    }

    let mut conn = connect_database(depot);

    //检查用户是否存在
    let existing_user = users::table
        .filter(users::username.eq(&register_req.username))
        .first::<User>(&mut conn)
        .optional()
        .expect("查询用户失败");

    if existing_user.is_some() {
        return ApiOut::err(AppError::BadRequest(
            format!("用户 '{}' 已经存在", register_req.username).to_string(),
        ));
    }

    //散列密码
    let hashed = match hash_password(register_req.password.clone().as_str()) {
        Ok(h) => h,
        Err(e) => {
            return ApiOut::err(AppError::BadRequest(
                format!("散列密码报错：{}", e).to_string(),
            ));
        }
    };
    let now = Local::now().naive_local();

    //创建用户
    let new_user = User {
        id: Uuid::new_v4(),
        username: register_req.username.clone(),
        password: hashed,
        full_name: register_req.username.clone(),
        create_time: now,
        update_time: now,
        access_token: "".to_string(),
        refresh_token: "".to_string(),
        is_delete: false,
    };

    //插入数据到数据库
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
        .expect("插入新用户失败！");

    ApiOut::ok(RegisterResp {
        username: register_req.username.to_string(),
        register_info: format!("用户'{}'创建成功！", register_req.username),
    })
}

#[endpoint(tags("Users"), summary = "登录", description = "登录",request_body = LoginReq)]
pub async fn login(depot: &mut Depot, req: &mut Request) -> ApiOut<LoginResp> {
    let login_req = match parse_json_body::<LoginReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    //验证验证码
    if let Err(e) = validate_captcha(depot, &login_req.captcha_id, &login_req.captcha_code).await {
        return ApiOut::err(e);
    }

    let mut conn = connect_database(depot);

    //连接数据库查询用户
    let existing_user = match users::table
        .filter(users::username.eq(&login_req.username))
        .first::<User>(&mut conn)
        .optional()
    {
        Ok(Some(user)) => user,
        Ok(None) => return ApiOut::err(AppError::BadRequest("未查询到该用户".to_string())),
        Err(e) => return ApiOut::err(AppError::Internal(e.to_string())),
    };

    if existing_user.is_delete == true {
        return ApiOut::err(AppError::BadRequest("当前用户已经被删除！".to_string()));
    }

    //验证密码
    if !verify_password(
        login_req.password.clone().as_str(),
        existing_user.password.clone().as_str(),
    ) {
        return ApiOut::err(AppError::BadRequest("密码错误".to_string()));
    }

    //创建1天access_token和7天refresh_token
    let user_id = existing_user.id.to_string();
    let username = existing_user.username.to_string();

    //转换String user_id 为Uuid
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            return ApiOut::err(AppError::UnAuthorized(format!("无效的用户ID格式: {}", e)));
        }
    };

    match generate_access_token(&user_id, &username) {
        Ok(access_token_str) => match generate_refresh_token(&user_id, &username) {
            Ok(refresh_token_str) => {
                //创建成功之后保存到数据库中//注意，后续可能要用redis存储，现在小项目无所谓
                match update_database_token(
                    user_uuid,
                    access_token_str,
                    refresh_token_str,
                    &mut conn,
                ) {
                    Ok(token_resp) => ApiOut::ok(LoginResp {
                        access_token: token_resp.access_token,
                        refresh_token: token_resp.refresh_token,
                        login_info: format!("用户'{}'登录成功！", login_req.username),
                    }),
                    Err(app_error) => ApiOut::Err(app_error),
                }
            }
            Err(e) => ApiOut::err(AppError::Internal(format!(
                "创建Refresh Token失败,请重试！'{}'",
                e
            ))),
        },
        Err(e) => ApiOut::err(AppError::Internal(format!("创建Token失败,请重试！'{}'", e))),
    }
}

#[endpoint(
    tags("Users"),
    summary = "获取用户信息",
    security(("Authorization" = [])),
    description = "获取用户信息"
)]
pub async fn get_users_info(depot: &mut Depot) -> ApiOut<UserInfoResp> {
    let current_user = depot.get::<User>("user").expect("未找到用户。");

    let user_response_model = UserInfoResp {
        id: current_user.id,
        full_name: current_user.full_name.clone(),
        username: current_user.username.clone(),
        create_time: current_user.create_time,
        is_delete: current_user.is_delete,
    };

    ApiOut::ok(user_response_model)
}

#[endpoint(tags("Users"),  summary = "刷新Token", description = "刷新Token",request_body = RefreshTokenReq
)]
pub async fn refresh_token(req: &mut Request, depot: &mut Depot) -> ApiOut<TokenResp> {
    let refresh_req = match parse_json_body::<RefreshTokenReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    let mut conn = connect_database(depot);

    let user_uuid = match Uuid::parse_str(&refresh_req.user_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            return ApiOut::err(AppError::UnAuthorized(format!("无效的用户ID格式: {}", e)));
        }
    };

    match users::table
        .filter(users::id.eq(user_uuid))
        .filter(users::refresh_token.eq(&refresh_req.refresh_token))
        .first::<User>(&mut conn)
        .optional()
    {
        Ok(Some(_)) => match refresh_access_token(&refresh_req.refresh_token) {
            Ok(refresh_token_resp) => {
                match update_database_token(
                    user_uuid,
                    refresh_token_resp.access_token,
                    refresh_token_resp.refresh_token,
                    &mut conn,
                ) {
                    Ok(token_resp) => ApiOut::ok(TokenResp {
                        access_token: token_resp.access_token,
                        refresh_token: token_resp.refresh_token,
                    }),
                    Err(app_error) => ApiOut::Err(app_error),
                }
            }
            Err(e) => ApiOut::err(AppError::UnAuthorized(format!(
                "刷新Token无效,请重新登录!{:?}",
                e
            ))),
        },
        Ok(None) => ApiOut::err(AppError::UnAuthorized(
            "未查询到刷新Token或刷新Token不一致,请重新登录!".to_string(),
        )),
        Err(e) => ApiOut::err(AppError::Internal(e.to_string())),
    }
}

//更新数据库中的token,用在登录和刷新Token中
fn update_database_token(
    user_uuid: Uuid,
    access_token_str: String,
    refresh_token_str: String,
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
) -> Result<TokenResp, AppError> {
    let result = diesel::update(users::table.find(user_uuid))
        .set((
            users::access_token.eq(&access_token_str),
            users::refresh_token.eq(&refresh_token_str),
        ))
        .execute(conn);

    match result {
        Ok(affected_rows) => {
            if affected_rows == 0 {
                Err(AppError::Internal(format!(
                    "未找到对应的user_id{}",
                    user_uuid
                )))
            } else {
                let token_resp = TokenResp {
                    access_token: access_token_str,
                    refresh_token: refresh_token_str,
                };
                Ok(token_resp)
            }
        }
        Err(e) => Err(AppError::Internal(format!(
            "更新保存Token失败,请重试！'{}'",
            e
        ))),
    }
}

//验证Token
#[endpoint(tags("Users"), security((
    "Authorization" = [])), summary = "验证Token", description = "验证Token")]
pub fn auth_token(depot: &mut Depot, ctrl: &mut FlowCtrl) -> ApiOut<()> {
    let auth_state = depot.jwt_auth_state();
    let token_data = depot.jwt_auth_data::<AccessTokenClaims>().cloned();
    let auth_error = depot.jwt_auth_error();
    let auth_token_owned = depot.jwt_auth_token().map(|s| s.to_string());

    match auth_state {
        JwtAuthState::Authorized => {
            info!("Token 有效");
            match token_data {
                None => {
                    ctrl.skip_rest();
                    ApiOut::err(AppError::UnAuthorized("Token数据未找到!".to_string()))
                }
                Some(token_data) => {
                    //验证Token是否过期
                    let current_timestamp = Local::now().naive_local().timestamp();
                    if token_data.claims.exp < current_timestamp {
                        return ApiOut::err(AppError::FORBIDDEN("Token过期".to_string()));
                    }

                    //Token 有效 验证用户有效性
                    let mut conn = connect_database(depot);

                    let user_id_uuid = match Uuid::parse_str(&token_data.claims.user_id) {
                        Ok(uuid) => uuid,
                        Err(e) => {
                            ctrl.skip_rest();
                            return ApiOut::err(AppError::UnAuthorized(format!(
                                "无效的用户ID格式: {}",
                                e
                            )));
                        }
                    };

                    let existing_user = users::table
                        .filter(users::username.eq(&token_data.claims.user_name))
                        .filter(users::id.eq(user_id_uuid))
                        .first::<User>(&mut conn)
                        .optional()
                        .expect("未找到该用户");

                    if let Some(user) = existing_user {
                        //连接数据库验证access_token是否存在或相等
                        if let Some(ref token) = auth_token_owned {
                            match users::table
                                .filter(users::id.eq(user_id_uuid))
                                .filter(users::access_token.eq(token))
                                .first::<User>(&mut conn)
                                .optional()
                            {
                                Ok(Some(_)) => {
                                    //验证通过则插入用户信息
                                    depot.insert("user", user);
                                    //验证通过则插入验证完成
                                    ApiOut::Ok(ApiResponse {
                                        data: None,
                                        code: 0,
                                        msg: "".to_string(),
                                    })
                                }
                                Ok(None) => ApiOut::err(AppError::UnAuthorized(
                                    "数据库Token不一致！".to_string(),
                                )),
                                Err(e) => ApiOut::err(AppError::UnAuthorized(format!(
                                    "数据库查询错误: {}",
                                    e
                                ))),
                            }
                        } else {
                            ApiOut::err(AppError::UnAuthorized(format!(
                                "用户'{}'未找到!)",
                                token_data.claims.user_name
                            )))
                        }
                    } else {
                        ctrl.skip_rest();
                        ApiOut::err(AppError::UnAuthorized(format!(
                            "用户'{}'未找到!)",
                            token_data.claims.user_name
                        )))
                    }
                }
            }
        }
        JwtAuthState::Unauthorized => {
            ctrl.skip_rest();
            ApiOut::err(AppError::UnAuthorized(
                "未找到Token，请检查Header".to_string(),
            ))
        }
        JwtAuthState::Forbidden => {
            ctrl.skip_rest();
            match auth_error {
                None => ApiOut::err(AppError::FORBIDDEN("Token报错为None".to_string())),
                Some(jwt_error) => {
                    ApiOut::err(AppError::FORBIDDEN(format!("Token无效,{}", jwt_error)))
                }
            }
        }
    }
}

//验证验证码
async fn validate_captcha(
    depot: &mut Depot,
    captcha_id: &str,
    captcha_code: &str,
) -> Result<(), AppError> {
    // 从depot中获取验证码缓存
    let cache = match depot.obtain::<Arc<Cache<String, String>>>() {
        Ok(cache) => cache,
        Err(_) => {
            return Err(AppError::Internal("验证码缓存未初始化".to_string()));
        }
    };

    // 根据验证码ID获取缓存中的验证码文本
    let cached = cache.get(captcha_id).await;
    let cached = match cached {
        Some(v) => v,
        None => {
            return Err(AppError::BadRequest("验证码已过期或不存在".to_string()));
        }
    };

    // 忽略大小写比对验证码
    if cached.trim().to_lowercase() != captcha_code.trim().to_lowercase() {
        return Err(AppError::BadRequest("验证码错误".to_string()));
    }

    // 验证码验证通过后删除缓存，防止复用
    cache.invalidate(captcha_id).await;

    // 验证通过，返回Ok
    Ok(())
}

pub fn users_router() -> Router {
    Router::with_path("users")
        .push(Router::with_path("get_auth_captcha").post(get_auth_captcha))
        .push(Router::with_path("register").post(register))
        .push(Router::with_path("login").post(login))
        .push(
            Router::with_path("get_users_info")
                .hoop(auth_token)
                .post(get_users_info),
        )
        .push(Router::with_path("refresh_token").post(refresh_token))
}
