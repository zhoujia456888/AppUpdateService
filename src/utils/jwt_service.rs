use crate::model::jwt::{AccessTokenClaims, RefreshTokenClaims, RefreshTokenResp, TokenType, JWT_CONFIG};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use time::OffsetDateTime;
// JWT工具函数

//创建访问令牌
pub fn generate_access_token(
    user_id: &str,
    user_name: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::seconds(JWT_CONFIG.access_expires_in);

    let claims = AccessTokenClaims {
        user_name: user_name.to_string(),
        user_id: user_id.to_string(),
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
        token_type: TokenType::Access,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_CONFIG.access_secret.as_bytes()),
    )
}

//创建刷新令牌
pub fn generate_refresh_token(
    user_id: &str,
    user_name: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::seconds(JWT_CONFIG.refresh_expires_in);

    let claims = RefreshTokenClaims {
        user_name: user_name.to_string(),
        user_id: user_id.to_string(),
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
        token_type: TokenType::Refresh,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_CONFIG.refresh_secret.as_bytes()),
    )
}
//刷新访问令牌
pub fn refresh_access_token(
    refresh_token: &str,
) -> Result<RefreshTokenResp, jsonwebtoken::errors::Error> {
    let refresh_token_claims = verify_refresh_token(refresh_token)?;

    let new_access_token = generate_access_token(&refresh_token_claims.user_id, &refresh_token_claims.user_name)?;
    let new_refresh_token = generate_refresh_token(&refresh_token_claims.user_id, &refresh_token_claims.user_name)?;

    Ok(RefreshTokenResp {
        claims:refresh_token_claims,
        access_token: new_access_token,
        refresh_token: new_refresh_token,
    })
}

//验证访问令牌
pub fn verify_access_token(token: &str) -> Result<AccessTokenClaims, jsonwebtoken::errors::Error> {
    let token_data = decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_secret(JWT_CONFIG.access_secret.as_bytes()),
        &Validation::default(),
    )?;

    match token_data.claims.token_type {
        TokenType::Access => Ok(token_data.claims),
        TokenType::Refresh => Err(jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken)),
    }
}

pub fn verify_refresh_token(
    token: &str,
) -> Result<RefreshTokenClaims, jsonwebtoken::errors::Error> {
    let token_data = decode::<RefreshTokenClaims>(
        token,
        &DecodingKey::from_secret(JWT_CONFIG.refresh_secret.as_bytes()),
        &Validation::default(),
    )?;

    //验证刷新token是否过期
    let current_timestamp = OffsetDateTime::now_utc().unix_timestamp();
    if token_data.claims.exp < current_timestamp {
        return Err(jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken));
    }

    match token_data.claims.token_type {
        TokenType::Refresh => Ok(token_data.claims),
        TokenType::Access => Err(jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken)),
    }
}
