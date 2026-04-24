use crate::model::users::AuthCaptcha;
use captcha_rs::CaptchaBuilder;
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
