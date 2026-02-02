use crate::model::error::{ApiOut, AppError};
use crate::model::users::{AuthCaptcha, CaptchaResp};
use captcha_rs::CaptchaBuilder;
use moka::future::Cache;
use salvo::Depot;
use std::sync::Arc;
use uuid::Uuid;

///
/// 生成登陆验证码
///
pub fn get_auth_captcha() -> AuthCaptcha {
    let captcha = CaptchaBuilder::new()
        .length(4)
        .width(130)
        .height(40)
        .dark_mode(false)
        .complexity(1) // min: 1, max: 10
        .compression(40) // min: 1, max: 99
        .build();

    let text = String::from(captcha.text.as_str());
    let base64 = captcha.to_base64();
    let id = Uuid::new_v4().to_string();

    AuthCaptcha {
        id,
        text,
        img: base64,
    }
}


//从内存中获取验证码缓存
pub fn get_captcha_cache(
    depot: &mut Depot,
) -> Result<&Arc<Cache<String, String>>, ApiOut<CaptchaResp>> {
    let cache = match depot.obtain::<Arc<Cache<String, String>>>() {
        Ok(cache) => cache,
        Err(_) => {
            return Err(ApiOut::err(AppError::Internal(
                "验证码缓存未初始化".to_string(),
            )));
        }
    };
    Ok(cache)
}