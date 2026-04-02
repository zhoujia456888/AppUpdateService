use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};

///上传文件返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadAppFileResp {
    ///文件路径
    pub file_path: String,
    ///APK文件名称
    pub apk_name: String,
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
