use crate::model::error::{ApiOut, AppError};
use crate::model::operation_log::{GetOperationLogListResp, OperationLog, OperationLogRespItem};
use crate::model::users::User;
use crate::schema::*;
use crate::utils::database_utils::connect_database;
use diesel::RunQueryDsl;
use diesel::prelude::*;
use salvo::prelude::*;

#[endpoint(
    tags("operation_log"),
    summary = "首页最近操作记录",
    description = "获取当前用户最近20条操作记录"
)]
pub async fn get_recent_operation_logs(depot: &mut Depot) -> ApiOut<GetOperationLogListResp> {
    let current_user = depot.get::<User>("user").expect("未找到用户。");
    let current_user_id = current_user.id;
    let mut conn = connect_database(depot);

    let operation_logs = match operation_log::table
        .filter(operation_log::user_id.eq(current_user_id))
        .order(operation_log::create_time.desc())
        .limit(20)
        .load::<OperationLog>(&mut conn)
    {
        Ok(logs) => logs,
        Err(e) => return ApiOut::err(AppError::Internal(format!("获取操作记录失败:{}", e))),
    };

    let total_count = match operation_log::table
        .filter(operation_log::user_id.eq(current_user_id))
        .count()
        .get_result::<i64>(&mut conn)
    {
        Ok(count) => count,
        Err(e) => return ApiOut::err(AppError::Internal(format!("获取操作记录总数失败:{}", e))),
    };

    let operation_logs = operation_logs
        .into_iter()
        .map(|log| OperationLogRespItem {
            id: log.id,
            username: log.username,
            operation_type: log.operation_type,
            operation_detail: log.operation_detail,
            create_time: log.create_time,
        })
        .collect();

    ApiOut::ok(GetOperationLogListResp {
        operation_logs,
        total_count,
    })
}

pub fn operation_log_router() -> Router {
    Router::with_path("operation_log")
        .push(Router::with_path("get_recent_operation_logs").post(get_recent_operation_logs))
}
