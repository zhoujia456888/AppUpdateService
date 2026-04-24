mod captcha_store;
mod token_store;

pub use captcha_store::{CaptchaStore, PostgresCaptchaStore};
pub use token_store::{PostgresTokenStore, TokenStore};

use crate::model::error::AppError;
use salvo::Depot;
use std::sync::Arc;

pub fn get_captcha_store(depot: &mut Depot) -> Result<Arc<dyn CaptchaStore>, AppError> {
    depot
        .obtain::<Arc<dyn CaptchaStore>>()
        .cloned()
        .map_err(|_| AppError::Internal("验证码存储未初始化".to_string()))
}

pub fn get_token_store(depot: &mut Depot) -> Result<Arc<dyn TokenStore>, AppError> {
    depot
        .obtain::<Arc<dyn TokenStore>>()
        .cloned()
        .map_err(|_| AppError::Internal("Token存储未初始化".to_string()))
}
