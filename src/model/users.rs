use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use salvo::macros::Extractible;
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

///数据库User表结构字段
#[derive(Queryable, Insertable, Serialize, Deserialize, Debug, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(sql_type=Timestamp)]
pub struct User {
    ///用户UUID
    pub id: Uuid,
    ///用户姓名
    pub username: String,
    ///用户密码
    pub password: String,
    ///用户全称
    pub full_name: String,
    ///用户访问Token
    pub access_token: String,
    ///用户刷新Token
    pub refresh_token: String,
    ///用户创建时间
    pub create_time: NaiveDateTime,
    ///是否删除
    pub is_delete: bool,
}

///创建用户请求参数
#[derive(Serialize, Deserialize, Extractible, Debug, ToSchema)]
#[salvo(extract(default_source(from = "body")))]
pub struct UserCreateReq {
    ///用户名
    pub username: String,
    ///密码
    pub password: String,
}

///创建用户返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCreateResp {
    ///用户名
    pub username: String,
    ///创建用户信息
    pub create_info: String,
}

///图片验证码参数
#[derive(Debug, Clone, Serialize)]
pub struct AuthCaptcha {
    /// 验证码ID
    pub id: String,
    /// 验证码文本
    pub text: String,
    /// 验证码图片 base64 编码
    pub img: String,
}

///图片验证码返回参数
#[derive(Serialize, Deserialize, Extractible, Debug, ToSchema)]
pub struct CaptchaResp {
    ///验证码ID
    pub captcha_id: String,
    ///验证码图片 base64
    pub captcha_img: String,
}

///用户登录请求参数
#[derive(Serialize, Deserialize, Extractible, Debug, ToSchema)]
#[salvo(extract(default_source(from = "body")))]
pub struct LoginReq {
    ///用户名
    pub username: String,
    ///密码
    pub password: String,
    ///验证码Id
    pub captcha_id: String,
    ///验证码code
    pub captcha_code: String,
}

///用户登录返回数据
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginResp {
    ///访问Token
    pub access_token: String,
    ///刷新Token
    pub refresh_token: String,
    ///登录信息
    pub login_info: String,
}

///用户信息返回数据
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserInfoResp {
    ///用户UUID
    pub id: Uuid,
    ///用户名
    pub username: String,
    ///用户全称
    pub full_name: String,
    ///创建时间
    pub create_time: NaiveDateTime,
    ///是否被删除
    pub is_delete: bool,
}

///测试获取用户请求参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetUserReq {
    ///用户Id
    pub id: i64,
}

///测试用户返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetUserResp {
    ///用户名称
    pub name: String,
}
