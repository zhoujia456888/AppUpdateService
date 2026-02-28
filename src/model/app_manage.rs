use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};

///上传文件返回参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadAppFileResp {
    ///文件路径
    pub file_path: String,
    ///上传文件信息
    pub upload_file_info: String,
}
