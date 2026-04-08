use crate::model::app_manage::{
    AppManage, UploadAppFileCompleteReq, UploadAppFileCompleteResp, UploadAppFileResp,
};
use crate::model::body::parse_json_body;
use crate::model::error::{ApiOut, AppError};
use crate::model::users::User;
use crate::schema::*;
use crate::utils::apk_utils::extract_apk_metadata;
use crate::utils::database_utils::connect_database;
use chrono::Local;
use diesel::RunQueryDsl;
use salvo::oapi::extract::FormFile;
use salvo::prelude::*;
use std::path::Path;
use uuid::Uuid;

const APP_MANAGE_DIR: &str = "app_manage";
const APK_UPLOAD_DIR: &str = "app_manage/apk";
const PUBLIC_APP_MANAGE_PREFIX: &str = "/api/public/app_manage";

#[endpoint(
    tags("app_manage"),
    summary = "上传APP文件",
    description = "上传APP文件"
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
            "未找到上传文件字段，请使用 multipart/form-data 并传入 file（兼容 upload_file/app_file）"
                .to_string(),
        ));
    };

    let file = FormFile::new(file_part);
    let raw_name = file.name().unwrap_or("file.bin");
    let filename = Path::new(raw_name)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("file.bin");
    let timestamp = Local::now().format("%Y%m%d%H%M%S%3f").to_string();
    let stamped_filename = build_timestamped_filename(filename, &timestamp);
    let dest = format!("{}/{}", APK_UPLOAD_DIR, stamped_filename);

    println!("upload: {}", dest);

    if let Err(e) = std::fs::create_dir_all(APK_UPLOAD_DIR) {
        return ApiOut::err(AppError::Internal(format!("创建上传目录失败: {}", e)));
    }

    if let Err(e) = std::fs::copy(file.path(), &dest) {
        ApiOut::err(AppError::Internal(format!("文件上传失败: {}", e)))
    } else {
        let apk_metadata = match extract_apk_metadata(
            Path::new(&dest),
            &stamped_filename,
            Path::new(APP_MANAGE_DIR),
        ) {
            Ok(metadata) => metadata,
            Err(e) => {
                let _ = std::fs::remove_file(&dest);
                return ApiOut::err(AppError::Unprocessable(format!("APK 解析失败: {}", e)));
            }
        };

        ApiOut::ok(UploadAppFileResp {
            file_path: to_public_app_manage_file_url("apk", &dest),
            file_name: apk_metadata.file_name,
            app_name: apk_metadata.app_name,
            package_name: apk_metadata.package_name,
            app_icon_path: apk_metadata
                .app_icon_path
                .as_deref()
                .map(|path| to_public_app_manage_file_url("icon", path)),
            version_name: apk_metadata.version_name,
            version_code: apk_metadata.version_code,
            file_size: apk_metadata.file_size,
            upload_file_info: "文件上传成功！".to_string(),
        })
    }
}

//组装带时间格式的APP文件名称
fn build_timestamped_filename(filename: &str, timestamp: &str) -> String {
    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("file");
    let extension = path.extension().and_then(|value| value.to_str());

    match extension.filter(|value| !value.trim().is_empty()) {
        Some(extension) => format!("{stem}_{timestamp}.{extension}"),
        None => format!("{stem}_{timestamp}"),
    }
}

// 组装公共APP管理文件URL
fn to_public_app_manage_file_url(kind: &str, path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let filename = normalized
        .rsplit('/')
        .next()
        .unwrap_or(normalized.as_str())
        .trim();

    format!("{PUBLIC_APP_MANAGE_PREFIX}/{kind}?name={filename}")
}

#[endpoint(tags("app_manage"), summary = "发布应用", description = "发布应用")]
pub async fn upload_app_file_complete(
    depot: &mut Depot,
    req: &mut Request,
) -> ApiOut<UploadAppFileCompleteResp> {
    let get_upload_app_file_complete_req =
        match parse_json_body::<UploadAppFileCompleteReq>(req).await {
            Ok(v) => v,
            Err(e) => return ApiOut::err(e),
        };

    if get_upload_app_file_complete_req.file_name.is_empty() {
        return ApiOut::err(AppError::BadRequest("文件不能为空".to_string()));
    }
    if get_upload_app_file_complete_req.app_name.is_empty() {
        return ApiOut::err(AppError::BadRequest("应用名称不能为空".to_string()));
    }
    if get_upload_app_file_complete_req.package_name.is_empty() {
        return ApiOut::err(AppError::BadRequest("包名不能为空".to_string()));
    }
    if get_upload_app_file_complete_req.file_path.is_empty() {
        return ApiOut::err(AppError::BadRequest("文件路径不能为空".to_string()));
    }

    let mut conn = connect_database(depot);
    let current_user_id = depot.get::<User>("user").expect("未找到用户。").id;

    let now = Local::now().naive_local();

    let new_app = AppManage {
        id: Uuid::new_v4(),
        app_name: get_upload_app_file_complete_req.app_name.clone(),
        app_download_url: get_upload_app_file_complete_req.file_path.clone(),
        create_user_id: current_user_id,
        create_time: now,
        update_time: now,
        is_delete: false,
        file_path: get_upload_app_file_complete_req.file_path.clone(),
        file_name: get_upload_app_file_complete_req.file_name.clone(),
        package_name: get_upload_app_file_complete_req.package_name.clone(),
        app_icon_path: get_upload_app_file_complete_req.app_icon_path.clone(),
        version_name: get_upload_app_file_complete_req.version_name.clone(),
        version_code: get_upload_app_file_complete_req.version_code.clone(),
        file_size: get_upload_app_file_complete_req.file_size.clone(),
        channel_name: get_upload_app_file_complete_req.channel_name.clone(),
        channel_id: get_upload_app_file_complete_req.channel_id.clone(),
        update_log: get_upload_app_file_complete_req.update_log.clone(),
    };

    match diesel::insert_into(app_manage::table)
        .values(&new_app)
        .execute(&mut conn)
    {
        Ok(_) => ApiOut::ok(UploadAppFileCompleteResp {
            upload_app_complete_info: format!(
                "应用'{}' (版本：{}) 发布成功！",
                get_upload_app_file_complete_req.app_name,
                get_upload_app_file_complete_req.version_name
            ),
        }),
        Err(e) => ApiOut::err(AppError::Internal(format!("保存应用信息失败：{}", e))),
    }
}

pub fn app_manage_router() -> Router {
    Router::with_path("app_manage")
        .push(Router::with_path("upload_app_file").post(upload_app_file))
        .push(Router::with_path("upload_app_file_complete").post(upload_app_file_complete))
}
