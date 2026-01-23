use crate::model::jwt::{JwtClaims, JwtConfig, TokenResponse, TokenType};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

#[derive(Clone)]
pub struct JwtService {
    config: JwtConfig,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    // 生成访问令牌
    pub fn generate_access_token(
        &self,
        user_id: &str,
        username: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let exp = now + Duration::days(self.config.access_expiration_days);

        let claims = JwtClaims {
            user_id: user_id.to_string(),
            username: username.to_string(),
            token_type: TokenType::Access,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.access_secret.as_bytes()),
        )
    }

    // 生成刷新令牌
    pub fn generate_refresh_token(
        &self,
        user_id: &str,
        username: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let exp = now + Duration::days(self.config.refresh_expiration_days);

        let claims = JwtClaims {
            user_id: user_id.to_string(),
            username: username.to_string(),
            token_type: TokenType::Refresh,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.refresh_secret.as_bytes()),
        )
    }

    // 验证访问令牌
    pub fn verify_access_token(
        &self,
        token: &str,
    ) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.config.access_secret.as_bytes()),
            &Validation::default(),
        )?;

        // 检查 token 类型
        match token_data.claims.token_type {
            TokenType::Access => Ok(token_data.claims),
            TokenType::Refresh => Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            )),
        }
    }

    // 验证刷新令牌
    pub fn verify_refresh_token(
        &self,
        token: &str,
    ) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.config.refresh_secret.as_bytes()),
            &Validation::default(),
        )?;

        // 检查 token 类型
        match token_data.claims.token_type {
            TokenType::Refresh => Ok(token_data.claims),
            TokenType::Access => Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            )),
        }
    }

    // 刷新访问令牌
    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, jsonwebtoken::errors::Error> {
        let claims = self.verify_refresh_token(refresh_token)?;

        let new_access_token = self.generate_access_token(&claims.user_id, &claims.username)?;
        let new_refresh_token = self.generate_refresh_token(&claims.user_id, &claims.username)?;

        Ok(TokenResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_expiration_days * 24 * 3600, // 转换为秒
        })
    }
}
