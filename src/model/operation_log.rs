use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Insertable, Serialize, Deserialize, Debug, Selectable)]
#[diesel(table_name = operation_log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(sql_type=Timestamp)]
pub struct OperationLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub operation_type: String,
    pub operation_detail: String,
    pub create_time: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetOperationLogListResp {
    pub operation_logs: Vec<OperationLogRespItem>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OperationLogRespItem {
    pub id: Uuid,
    pub username: String,
    pub operation_type: String,
    pub operation_detail: String,
    pub create_time: NaiveDateTime,
}
