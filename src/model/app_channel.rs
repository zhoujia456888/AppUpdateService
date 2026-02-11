use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use salvo::macros::Extractible;
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

///数据库应用渠道AppChannel表结构字段
#[derive(Queryable, Insertable, Serialize, Deserialize, Debug, Selectable)]
#[diesel(table_name = app_channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(sql_type=Timestamp)]
pub struct AppChannel {
    ///渠道UUID
    pub id: Uuid,
    ///渠道名称
    pub channel_name: String,
    ///创建者id
    pub create_user_id: Uuid,
    ///用户创建时间
    pub create_time: NaiveDateTime,
    ///渠道更新时间
    pub update_time: NaiveDateTime,
    ///是否删除
    pub is_delete: bool,
}

///创建应用渠道请求参数
#[derive(Serialize, Deserialize, Extractible, Debug, ToSchema)]
#[salvo(extract(default_source(from = "body")))]
pub struct CreateAppChannelReq {
    ///渠道名称
    pub channel_name: String,
}

///创建应用渠道返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAppChannelResp {
    ///渠道名称
    pub channel_name: String,
    ///创建渠道信息
    pub create_info: String,
}

///分页查询渠道列表返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetAppChannelListReq {
    ///分页查询渠道列表大小
    pub page_size: i64,
    ///分页查询渠道列表索引
    pub page_index: i64,
}

///获取当前账户下的所有渠道返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetAppChannelListResp {
    pub channel_list: Vec<GetAppChannelListRespItem>,
    ///渠道总数
    pub total_channel_count: i64,
    ///总共页数
    pub total_page_count: i64,
}

///分页查询渠道列表返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetAppChannelListRespItem {
    ///渠道Id
    pub channel_id: Uuid,
    ///渠道名称
    pub channel_name: String,
    ///创建渠道时间
    pub create_time: NaiveDateTime,
    ///更新渠道时间
    pub update_time: NaiveDateTime,
}

///搜索渠道信息请求参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchAppChannelReq {
    ///渠道Id
    pub channel_id: Uuid,
    ///渠道名称
    pub channel_name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchAppChannelResp {
    pub channel_list: Vec<GetAppChannelListRespItem>,
    ///渠道总数
    pub total_channel_count: i64,
    ///总共页数
    pub total_page_count: i64,
}
///搜索渠道信息返回参数

///更新渠道信息请求参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateAppChannelReq {
    ///渠道Id
    pub channel_id: Uuid,
    /// 渠道名称
    pub channel_name: String,
}

///更新渠道信息返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateAppChannelResp {
    ///渠道Id
    pub channel_id: Uuid,
    /// 渠道名称
    pub channel_name: String,
    ///更新信息
    pub update_info: String,
}

///删除渠道
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeleteAppChannelReq {
    ///渠道Id
    pub channel_id: Uuid,
    /// 渠道名称
    pub channel_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeleteAppChannelResp {
    ///渠道Id
    pub channel_id: Uuid,
    ///删除信息
    pub delete_info: String,
}
