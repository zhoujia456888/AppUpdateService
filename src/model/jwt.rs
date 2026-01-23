use once_cell::sync::Lazy;
use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use std::env;

pub fn get_jwt_secret_key() -> String {
    let jwt_secret_key = env::var("JWT_SECRET_KEY").expect("要在env中设置JWT_SECRET_KEY！");
    jwt_secret_key
}

pub fn get_jwt_refresh_secret_key() -> String {
    let jwt_refresh_secret_key =
        env::var("JWT_REFRESH_SECRET_KEY").expect("要在env中设置JWT_REFRESH_SECRET_KEY！");
    jwt_refresh_secret_key
}

/// 配置
pub struct JwtConfig {
    pub access_secret: String,
    pub refresh_secret: String,
    pub access_expires_in: i64,  // 秒
    pub refresh_expires_in: i64, // 秒
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            access_secret: get_jwt_secret_key(),
            refresh_secret: get_jwt_refresh_secret_key(),
            access_expires_in: 24 * 60 * 60,      // 1天
            refresh_expires_in: 7 * 24 * 60 * 60, // 7天
        }
    }
}

pub static JWT_CONFIG: Lazy<JwtConfig> = Lazy::new(JwtConfig::default);

#[derive(Debug, Serialize, Clone, Deserialize, ToSchema)]
pub enum TokenType {
    Access,
    Refresh,
}

// Token Claims
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct AccessTokenClaims {
    pub user_name: String,     //用户名称
    pub user_id: String,       // 用户ID
    pub exp: i64,              // 过期时间
    pub iat: i64,              // 签发时间
    pub token_type: TokenType, // token类型
}

#[derive(Debug, Serialize, Clone, Deserialize, ToSchema)]
pub struct RefreshTokenClaims {
    pub user_name: String, //用户名称
    pub user_id: String,   // 用户ID
    pub exp: i64,
    pub iat: i64,
    pub token_type: TokenType,
}

//刷新Token携带的参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RefreshTokenReq {
    pub user_id: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct  RefreshTokenResp{
    pub claims: RefreshTokenClaims,
    pub access_token: String,
    pub refresh_token: String,
}

// Token 响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenResp{
    pub access_token: String,
    pub refresh_token: String,
}
