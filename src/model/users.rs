use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use salvo::macros::Extractible;
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Serialize, Deserialize, Debug, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(sql_type=Timestamp)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub full_name: String,
    pub access_token: String,
    pub refresh_token:String,
    pub create_time: NaiveDateTime,
    pub is_delete: bool,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub full_name: String,
    pub access_token: String,
    pub refresh_token:String,
    pub create_time: NaiveDateTime,
    pub is_delete: bool,
}

#[derive(Serialize, Deserialize, Extractible, Debug, ToSchema)]
#[salvo(extract(default_source(from = "body")))]
pub struct UserCreateReq {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCreateResp {
    pub username: String,
    pub create_info: String,
}

#[derive(Serialize, Deserialize, Extractible, Debug, ToSchema)]
#[salvo(extract(default_source(from = "body")))]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginResp {
    pub access_token: String,
    pub refresh_token:String,
    pub login_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenAuthResp {
    pub user_name: String,
    pub user_id:String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserInfoResp {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub create_time: NaiveDateTime,
    pub is_delete: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct  GetUserInfoReq{
    pub id: String,
    pub username: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetUserReq {
    pub id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetUserResp {
    pub name: String,
}
