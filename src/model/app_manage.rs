use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

///数据库应用AppManage表结构字段
#[derive(Queryable, Insertable, Serialize, Deserialize, Debug, Selectable)]
#[diesel(table_name = app_manage)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(sql_type=Timestamp)]
pub struct AppManage {
    ///渠道UUID
    pub id: Uuid,
    ///应用名称
    pub app_name: String,
    ///应用下载地址
    pub app_download_url: String,
    ///创建人ID
    pub create_user_id: Uuid,
    ///渠道ID
    pub channel_id: Uuid,
    ///创建时间
    pub create_time: NaiveDateTime,
    ///更新时间
    pub update_time: NaiveDateTime,
    ///是否删除
    pub is_delete: bool,
    ///文件路径
    pub file_path: String,
    ///文件名称
    pub file_name: String,
    ///包名
    pub package_name: String,
    ///APP图标文件路径
    pub app_icon_path: String,
    ///版本名称
    pub version_name: String,
    ///版本号
    pub version_code: String,
    ///文件大小
    pub file_size: i64,
    ///渠道名称
    pub channel_name: String,
    ///更新日志
    pub update_log: String,
}

///上传文件返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadAppFileResp {
    ///文件路径
    pub file_path: String,
    ///文件名称
    pub file_name: String,
    ///APP名称
    pub app_name: String,
    ///包名
    pub package_name: String,
    ///APP图标文件路径
    pub app_icon_path: Option<String>,
    ///版本名称
    pub version_name: Option<String>,
    ///版本号
    pub version_code: Option<String>,
    ///文件大小（字节）
    pub file_size: u64,
    ///上传文件信息
    pub upload_file_info: String,
}

///完成应用发布请求参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadAppFileCompleteReq {
    ///文件路径
    pub file_path: String,
    ///文件名称
    pub file_name: String,
    ///APP名称
    pub app_name: String,
    ///包名
    pub package_name: String,
    ///APP图标文件路径
    pub app_icon_path: String,
    ///版本名称
    pub version_name: String,
    ///版本号
    pub version_code: String,
    ///文件大小（字节）
    pub file_size: i64,
    ///渠道ID
    pub channel_id: Uuid,
    ///渠道名称
    pub channel_name: String,
    ///更新日志
    pub update_log: String,
}

///完成应用发布返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadAppFileCompleteResp {
    ///发布应用信息
    pub upload_app_complete_info: String,
}
