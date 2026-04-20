use crate::model::app_manage::{
    AppCheckUpdateReq, AppCheckUpdateResp, AppManage, DeleteAppReq, DeleteAppResp, GetAppInfoReq,
    GetAppListReq, GetAppListResp, GetAppListRespItem, UploadAppFileCompleteReq,
    UploadAppFileCompleteResp, UploadAppFileResp,
};
use crate::model::body::parse_json_body;
use crate::model::error::{ApiOut, AppError};
use crate::model::users::User;
use crate::schema::*;
use crate::utils::apk_utils::extract_apk_metadata;
use crate::utils::database_utils::connect_database;
use crate::utils::operation_log_utils::{
    OP_DELETE_APP, OP_PUBLISH_APP, OP_UPLOAD_APP_FILE, record_operation,
};
use chrono::Local;
use diesel::RunQueryDsl;
use diesel::prelude::*;
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
pub async fn upload_app_file(depot: &mut Depot, req: &mut Request) -> ApiOut<UploadAppFileResp> {
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

        let current_user = depot.get::<User>("user").expect("未找到用户。");
        let current_user_id = current_user.id;
        let current_username = current_user.username.clone();
        let mut conn = connect_database(depot);
        if let Err(e) = record_operation(
            &mut conn,
            current_user_id,
            &current_username,
            OP_UPLOAD_APP_FILE,
            format!("上传应用文件'{}'成功", apk_metadata.file_name),
        ) {
            return ApiOut::err(e);
        }

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

/// 发布应用
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
    let current_user = depot.get::<User>("user").expect("未找到用户。");
    let current_user_id = current_user.id;
    let current_username = current_user.username.clone();

    let now = Local::now().naive_local();

    let new_app = AppManage {
        id: Uuid::new_v4(),
        app_name: get_upload_app_file_complete_req.app_name.clone(),
        app_download_url: get_upload_app_file_complete_req.file_path.clone(),
        create_user_id: current_user_id,
        create_time: now,
        update_time: now,
        is_delete: false,
        file_path: Some(get_upload_app_file_complete_req.file_path.clone()),
        file_name: Some(get_upload_app_file_complete_req.file_name.clone()),
        package_name: Some(get_upload_app_file_complete_req.package_name.clone()),
        app_icon_path: Some(get_upload_app_file_complete_req.app_icon_path.clone()),
        version_name: Some(get_upload_app_file_complete_req.version_name.clone()),
        version_code: get_upload_app_file_complete_req.version_code.clone(),
        file_size: get_upload_app_file_complete_req.file_size,
        channel_name: Some(get_upload_app_file_complete_req.channel_name.clone()),
        channel_id: get_upload_app_file_complete_req.channel_id,
        update_log: Some(get_upload_app_file_complete_req.update_log.clone()),
    };

    match diesel::insert_into(app_manage::table)
        .values(&new_app)
        .execute(&mut conn)
    {
        Ok(_) => {
            if let Err(e) = record_operation(
                &mut conn,
                current_user_id,
                &current_username,
                OP_PUBLISH_APP,
                format!(
                    "发布应用'{}'成功，版本：{}",
                    get_upload_app_file_complete_req.app_name,
                    get_upload_app_file_complete_req.version_name
                ),
            ) {
                return ApiOut::err(e);
            }

            ApiOut::ok(UploadAppFileCompleteResp {
                upload_app_complete_info: format!(
                    "应用'{}' (版本：{}) 发布成功！",
                    get_upload_app_file_complete_req.app_name,
                    get_upload_app_file_complete_req.version_name
                ),
            })
        }
        Err(e) => ApiOut::err(AppError::Internal(format!("保存应用信息失败：{}", e))),
    }
}

/// 分页查询应用列表
#[endpoint(tags("app_manage"), summary = "应用列表", description = "应用列表")]
pub async fn get_app_list_by_page(depot: &mut Depot, req: &mut Request) -> ApiOut<GetAppListResp> {
    let get_app_list_req = match parse_json_body::<GetAppListReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    if get_app_list_req.page_size <= 0 || get_app_list_req.page_index < 0 {
        return ApiOut::err(AppError::BadRequest(
            "分页参数错误，page_size必须大于0，page_index必须大于等于0".to_string(),
        ));
    }

    let user_id = depot.get::<User>("user").expect("未找到用户。").id;
    let mut conn = connect_database(depot);

    let keyword = get_app_list_req.search_key.trim();

    let all_user_apps = match app_manage::table
        .filter(app_manage::create_user_id.eq(&user_id))
        .filter(app_manage::is_delete.eq(false))
        .order(app_manage::create_time.desc())
        .load::<AppManage>(&mut conn)
    {
        Ok(list) => list,
        Err(e) => return ApiOut::err(AppError::Internal(format!("获取应用列表失败:{}", e))),
    };

    let filtered_apps: Vec<AppManage> = all_user_apps
        .into_iter()
        .filter(|app| matches_app_search(app, keyword))
        .collect();

    let total_app_count = filtered_apps.len() as i64;

    let total_page_count = if total_app_count == 0 {
        0
    } else {
        (total_app_count + get_app_list_req.page_size - 1) / get_app_list_req.page_size
    };

    let offset = (get_app_list_req.page_index * get_app_list_req.page_size) as usize;
    let limit = get_app_list_req.page_size as usize;
    let all_app_list: Vec<AppManage> = filtered_apps.into_iter().skip(offset).take(limit).collect();

    let app_list = all_app_list
        .into_iter()
        .map(|app| get_app_resp_item(&app))
        .collect();

    ApiOut::ok(GetAppListResp {
        app_list,
        total_app_count,
        total_page_count,
    })
}

fn matches_app_search(app: &AppManage, keyword: &str) -> bool {
    if keyword.is_empty() {
        return true;
    }

    app.app_name.contains(keyword)
        || app
            .package_name
            .as_deref()
            .unwrap_or_default()
            .contains(keyword)
        || app
            .channel_name
            .as_deref()
            .unwrap_or_default()
            .contains(keyword)
}

#[endpoint(
    tags("app_manage"),
    summary = "删除应用",
    description = "删除应用",
    request_body = DeleteAppReq
)]
pub async fn delete_app(depot: &mut Depot, req: &mut Request) -> ApiOut<DeleteAppResp> {
    let delete_app_req = match parse_json_body::<DeleteAppReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    if delete_app_req.app_name.trim().is_empty() {
        return ApiOut::err(AppError::BadRequest("应用名称不能为空".to_string()));
    }

    let mut conn = connect_database(depot);
    let current_user = depot.get::<User>("user").expect("未找到用户。");
    let user_id = current_user.id;
    let username = current_user.username.clone();

    let result = diesel::update(
        app_manage::table
            .filter(app_manage::id.eq(delete_app_req.app_id))
            .filter(app_manage::create_user_id.eq(user_id))
            .filter(app_manage::is_delete.eq(false)),
    )
    .set((
        app_manage::is_delete.eq(true),
        app_manage::update_time.eq(Local::now().naive_local()),
    ))
    .execute(&mut conn);

    match result {
        Ok(0) => ApiOut::err(AppError::NotFound(format!(
            "应用Id'{}' 未找到",
            delete_app_req.app_id
        ))),
        Ok(_) => {
            if let Err(e) = record_operation(
                &mut conn,
                user_id,
                &username,
                OP_DELETE_APP,
                format!("删除应用'{}'成功", delete_app_req.app_name),
            ) {
                return ApiOut::err(e);
            }

            ApiOut::ok(DeleteAppResp {
                app_id: delete_app_req.app_id,
                delete_info: format!("应用'{}'删除成功", delete_app_req.app_name),
            })
        }
        Err(e) => ApiOut::err(AppError::Internal(format!("删除应用失败:{}", e))),
    }
}

#[endpoint(
    tags("public"),
    summary = "应用详情",
    description = "根据应用ID查询应用详情"
)]
pub async fn get_app_info(depot: &mut Depot, req: &mut Request) -> ApiOut<GetAppListRespItem> {
    let get_app_info_req = match parse_json_body::<GetAppInfoReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    let mut conn = connect_database(depot);
    let app = match app_manage::table
        .filter(app_manage::is_delete.eq(false))
        .filter(app_manage::id.eq(get_app_info_req.app_id))
        .first::<AppManage>(&mut conn)
    {
        Ok(app) => app,
        Err(diesel::result::Error::NotFound) => {
            return ApiOut::err(AppError::NotFound("应用不存在".to_string()));
        }
        Err(e) => return ApiOut::err(AppError::Internal(format!("获取应用详情失败:{}", e))),
    };

    ApiOut::ok(get_app_resp_item(&app))
}

#[endpoint(
    tags("public"),
    summary = "检查应用更新",
    description = "根据包名和渠道查询最新应用版本",
    request_body = AppCheckUpdateReq
)]
pub async fn app_check_update(depot: &mut Depot, req: &mut Request) -> ApiOut<AppCheckUpdateResp> {
    let app_check_update_req = match parse_json_body::<AppCheckUpdateReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    if let Err(e) = validate_app_check_update_req(&app_check_update_req) {
        return ApiOut::err(e);
    }

    let mut conn = connect_database(depot);
    let app = match app_manage::table
        .filter(app_manage::is_delete.eq(false))
        .filter(app_manage::package_name.eq(Some(app_check_update_req.package_name.clone())))
        .filter(app_manage::channel_name.eq(Some(app_check_update_req.channel_name.clone())))
        .order((
            app_manage::create_time.desc(),
            app_manage::update_time.desc(),
        ))
        .first::<AppManage>(&mut conn)
    {
        Ok(app) => app,
        Err(diesel::result::Error::NotFound) => {
            return ApiOut::err(AppError::NotFound("未找到匹配的应用版本".to_string()));
        }
        Err(e) => return ApiOut::err(AppError::Internal(format!("检查应用更新失败:{}", e))),
    };

    ApiOut::ok(build_app_check_update_resp(&app))
}

// 校验应用更新请求参数
fn validate_app_check_update_req(app_check_update_req: &AppCheckUpdateReq) -> Result<(), AppError> {
    if app_check_update_req.package_name.trim().is_empty() {
        return Err(AppError::BadRequest("包名不能为空".to_string()));
    }

    if app_check_update_req.channel_name.trim().is_empty() {
        return Err(AppError::BadRequest("渠道不能为空".to_string()));
    }

    Ok(())
}

// 构建应用更新响应
fn build_app_check_update_resp(app: &AppManage) -> AppCheckUpdateResp {
    AppCheckUpdateResp {
        app_name: app.app_name.clone(),
        package_name: app.package_name.clone().unwrap_or_default(),
        channel_name: app.channel_name.clone().unwrap_or_default(),
        version_name: app.version_name.clone().unwrap_or_default(),
        version_code: app.version_code.clone(),
        app_download_url: app.app_download_url.clone(),
    }
}

// 应用详情响应项
fn get_app_resp_item(app: &AppManage) -> GetAppListRespItem {
    GetAppListRespItem {
        app_id: app.id,
        app_name: app.app_name.clone(),
        app_download_url: app.app_download_url.clone(),
        channel_id: app.channel_id,
        file_path: app.file_path.clone().unwrap_or_default(),
        file_name: app.file_name.clone().unwrap_or_default(),
        package_name: app.package_name.clone().unwrap_or_default(),
        app_icon_path: app.app_icon_path.clone().unwrap_or_default(),
        version_name: app.version_name.clone().unwrap_or_default(),
        version_code: app.version_code.clone(),
        file_size: app.file_size,
        channel_name: app.channel_name.clone().unwrap_or_default(),
        update_log: app.update_log.clone().unwrap_or_default(),
        create_time: app.create_time,
        update_time: app.update_time,
    }
}

pub fn app_manage_router() -> Router {
    Router::with_path("app_manage")
        .push(Router::with_path("upload_app_file").post(upload_app_file))
        .push(Router::with_path("upload_app_file_complete").post(upload_app_file_complete))
        .push(Router::with_path("get_app_list_by_page").post(get_app_list_by_page))
        .push(Router::with_path("delete_app").post(delete_app))
}
