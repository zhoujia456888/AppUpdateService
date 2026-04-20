use crate::model::error::AppError;
use crate::model::operation_log::OperationLog;
use crate::schema::operation_log;
use chrono::Local;
use diesel::PgConnection;
use diesel::RunQueryDsl;
use uuid::Uuid;

pub const OP_LOGIN: &str = "LOGIN";
pub const OP_UPLOAD_APP_FILE: &str = "UPLOAD_APP_FILE";
pub const OP_PUBLISH_APP: &str = "PUBLISH_APP";
pub const OP_CREATE_APP_CHANNEL: &str = "CREATE_APP_CHANNEL";
pub const OP_DELETE_APP_CHANNEL: &str = "DELETE_APP_CHANNEL";
pub const OP_DELETE_APP: &str = "DELETE_APP";

pub fn record_operation(
    conn: &mut PgConnection,
    user_id: Uuid,
    username: &str,
    operation_type: &str,
    operation_detail: String,
) -> Result<(), AppError> {
    let new_log = OperationLog {
        id: Uuid::new_v4(),
        user_id,
        username: username.to_string(),
        operation_type: operation_type.to_string(),
        operation_detail,
        create_time: Local::now().naive_local(),
    };

    diesel::insert_into(operation_log::table)
        .values(&new_log)
        .execute(conn)
        .map(|_| ())
        .map_err(|e| AppError::Internal(format!("记录操作日志失败:{}", e)))
}
