use crate::model::app_manage::UploadAppFileResp;
use crate::model::error::{ApiOut, AppError};
use salvo::prelude::*;
use salvo::oapi::extract::FormFile;
use std::path::Path;

#[endpoint(
    tags("app_manage"),
    summary = "上传APP文件",
    description = "上传APP文件",
)]
pub async fn upload_app_file(req: &mut Request) -> ApiOut<UploadAppFileResp> {
    println!("进入 upload_app_file 函数");
    // 默认安全上限仅 64KB，上传 APK 会在 multipart 解析阶段失败。
    // 上传文件大小上限设置为 1GB
    req.set_secure_max_size(1024 * 1024 * 1024);

    let file_part = match req.try_file("file").await {
        Ok(Some(f)) => Some(f),
        Ok(None) => match req.try_file("upload_file").await {
            Ok(Some(f)) => Some(f),
            Ok(None) => match req.try_file("app_file").await {
                Ok(Some(f)) => Some(f),
                Ok(None) => None,
                Err(e) => {
                    return ApiOut::err(AppError::BadRequest(format!(
                        "解析 multipart 失败（app_file）: {}。请不要手动设置 Content-Type，并确认使用 form-data file 类型字段",
                        e
                    )));
                }
            },
            Err(e) => {
                return ApiOut::err(AppError::BadRequest(format!(
                    "解析 multipart 失败（upload_file）: {}。请不要手动设置 Content-Type，并确认使用 form-data file 类型字段",
                    e
                )));
            }
        },
        Err(e) => {
            return ApiOut::err(AppError::BadRequest(format!(
                "解析 multipart 失败（file）: {}。请不要手动设置 Content-Type，并确认使用 form-data file 类型字段",
                e
            )));
        }
    };

    let Some(file_part) = file_part else {
        return ApiOut::err(AppError::BadRequest(
            "未找到上传文件字段，请使用 multipart/form-data 并传入 file（兼容 upload_file/app_file）".to_string(),
        ));
    };

    let file = FormFile::new(file_part);
    let raw_name = file.name().unwrap_or("file.bin");
    let filename = Path::new(raw_name)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("file.bin");
    let dest = format!("app_manage/{}", filename);

    println!("upload: {}", dest);

    if let Err(e) = std::fs::create_dir_all("app_manage") {
        return ApiOut::err(AppError::Internal(format!("创建上传目录失败: {}", e)));
    }

    if let Err(e) = std::fs::copy(file.path(), &dest) {
        ApiOut::err(AppError::Internal(format!("文件上传失败: {}", e)))
    } else {
        ApiOut::ok(UploadAppFileResp {
            file_path: dest,
            upload_file_info: "文件上传成功！".to_string(),
        })
    }
}

pub fn app_manage_router() -> Router {
    Router::with_path("app_manage").push(Router::with_path("upload_app_file").post(upload_app_file))
}
