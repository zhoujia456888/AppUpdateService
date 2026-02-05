use crate::api::users::auth_token;
use crate::model::app_channel::{
    AppChannel, CreateAppChannelReq, CreateAppChannelResp, DeleteAppChannelReq,
    DeleteAppChannelResp, GetAppChannelListResp, UpdateAppChannelReq, UpdateAppChannelResp,
};
use crate::model::body::parse_json_body;
use crate::model::error::{ApiOut, AppError};
use crate::model::users::User;
use crate::schema::*;
use crate::utils::database_utils::connect_database;
use chrono::{Local, Utc};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use salvo::prelude::*;
use salvo_oapi::endpoint;
use uuid::Uuid;

#[endpoint(tags("app_channel"), summary = "创建渠道", description = "创建渠道",request_body = CreateAppChannelReq
)]
pub async fn create_app_channel(
    depot: &mut Depot,
    req: &mut Request,
) -> ApiOut<CreateAppChannelResp> {
    let app_channel_create = match parse_json_body::<CreateAppChannelReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    let mut conn = connect_database(depot);

    let current_user_id = depot.get::<User>("user").expect("未找到用户。").id;

    //检查渠道是否存在
    let existing_app_channel = app_channel::table
        .filter(app_channel::channel_name.eq(&app_channel_create.channel_name))
        .filter(app_channel::create_user_id.eq(&current_user_id))
        .filter(app_channel::is_delete.eq(false))
        .first::<AppChannel>(&mut conn)
        .optional()
        .expect("查询渠道失败");

    if existing_app_channel.is_some() {
        return ApiOut::err(AppError::BadRequest(
            format!("渠道'{}' 已经存在", app_channel_create.channel_name).to_string(),
        ));
    }

    let now = Local::now().naive_local();

    //创建新渠道
    let new_channel = AppChannel {
        id: Uuid::new_v4(),
        channel_name: app_channel_create.channel_name.clone(),
        create_user_id: current_user_id,
        create_time: now,
        update_time: now,
        is_delete: false,
    };

    //插入数据到数据库
    diesel::insert_into(app_channel::table)
        .values(&new_channel)
        .execute(&mut conn)
        .expect("插入新渠道失败！");

    ApiOut::ok(CreateAppChannelResp {
        channel_name: app_channel_create.channel_name.to_string(),
        create_info: format!("渠道'{}'创建成功！", app_channel_create.channel_name),
    })
}

#[endpoint(
    tags("app_channel"),
    summary = "获取渠道列表",
    description = "获取渠道列表"
)]
pub async fn get_app_channel_list(depot: &mut Depot) -> ApiOut<Vec<GetAppChannelListResp>> {
    let user_id = depot.get::<User>("user").expect("未找到用户。").id;

    let mut conn = connect_database(depot);

    let all_app_channel = app_channel::table
        .filter(app_channel::create_user_id.eq(&user_id))
        .filter(app_channel::is_delete.eq(false))
        .load::<AppChannel>(&mut conn)
        .expect("获取当前用户的渠道数据失败");

    let resp_list: Vec<GetAppChannelListResp> = all_app_channel
        .into_iter()
        .map(|channel| GetAppChannelListResp {
            channel_id: channel.id,
            channel_name: channel.channel_name.to_string(),
            create_time: channel.create_time,
        })
        .collect();

    ApiOut::ok(resp_list)
}

#[endpoint(
    tags("app_channel"),
    summary = "更新渠道信息",
    description = "更新渠道信息",request_body = UpdateAppChannelReq
)]
pub async fn update_app_channel(
    depot: &mut Depot,
    req: &mut Request,
) -> ApiOut<UpdateAppChannelResp> {
    let app_channel_req = match parse_json_body::<UpdateAppChannelReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    let mut conn = connect_database(depot);

    let result = diesel::update(app_channel::table.find(app_channel_req.id))
        .set((
            app_channel::channel_name.eq(&app_channel_req.channel_name),
            app_channel::update_time.eq(&Utc::now().naive_utc()),
            app_channel::is_delete.eq(false),
        ))
        .execute(&mut conn);

    match result {
        Ok(affected_rows) => {
            if affected_rows == 0 {
                ApiOut::err(AppError::NotFound(
                    format!("渠道Id'{}' 未找到", app_channel_req.id).to_string(),
                ))
            } else {
                ApiOut::ok(UpdateAppChannelResp {
                    id: app_channel_req.id,
                    channel_name: app_channel_req.channel_name,
                    update_info: "更新渠道信息成功".to_string(),
                })
            }
        }
        Err(e) => ApiOut::err(AppError::Internal(
            format!("更新渠道信息失败:{}", e).to_string(),
        )),
    }
}

#[endpoint(
    tags("app_channel"),
    summary = "删除渠道",
    description = "删除渠道",request_body = DeleteAppChannelReq
)]
pub async fn delete_app_channel(
    depot: &mut Depot,
    req: &mut Request,
) -> ApiOut<DeleteAppChannelResp> {
    let app_channel_req = match parse_json_body::<DeleteAppChannelReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    let mut conn = connect_database(depot);

    let result = diesel::update(app_channel::table.find(app_channel_req.id))
        .set((app_channel::is_delete.eq(&true),))
        .execute(&mut conn);

    match result {
        Ok(affected_rows) => {
            if affected_rows == 0 {
                ApiOut::err(AppError::NotFound(
                    format!("渠道Id'{}' 未找到", app_channel_req.id).to_string(),
                ))
            } else {
                ApiOut::ok(DeleteAppChannelResp {
                    id: app_channel_req.id,
                    delete_info: format!("渠道'{}'删除成功", app_channel_req.channel_name),
                })
            }
        }
        Err(e) => ApiOut::err(AppError::Internal(
            format!("删除渠道信息失败:{}", e).to_string(),
        )),
    }
}

#[endpoint(
    tags("app_channel"),
    summary = "完全删除渠道",
    description = "完全删除渠道",request_body = DeleteAppChannelReq
)]
pub async fn completely_delete_app_channel(
    depot: &mut Depot,
    req: &mut Request,
) -> ApiOut<DeleteAppChannelResp> {
    let app_channel_req = match parse_json_body::<DeleteAppChannelReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    let mut conn = connect_database(depot);

    let affected =
        diesel::delete(app_channel::table.filter(app_channel::id.eq(app_channel_req.id)))
            .execute(&mut conn);

    match affected {
        Ok(0) => ApiOut::err(AppError::NotFound(
            format!("渠道Id'{}' 未找到", app_channel_req.id).to_string(),
        )),
        Ok(affected_rows) => ApiOut::ok(DeleteAppChannelResp {
            id: app_channel_req.id,
            delete_info: format!("渠道'{}'删除成功", app_channel_req.channel_name),
        }),
        Err(e) => ApiOut::err(AppError::Internal(
            format!("完全删除渠道信息失败:{}", e).to_string(),
        )),
    }
}

pub fn app_channel_router() -> Router {
    Router::with_path("app_channel")
        .push(
            Router::with_path("create_app_channel")
                .hoop(auth_token)
                .post(create_app_channel),
        )
        .push(
            Router::with_path("get_app_channel_list")
                .hoop(auth_token)
                .post(get_app_channel_list),
        )
        .push(
            Router::with_path("update_app_channel")
                .hoop(auth_token)
                .post(update_app_channel),
        )
        .push(
            Router::with_path("delete_app_channel")
                .hoop(auth_token)
                .post(delete_app_channel),
        )
        .push(
            Router::with_path("completely_delete_app_channel")
                .hoop(auth_token)
                .post(completely_delete_app_channel),
        )
}
