use crate::db::DbPool;
use crate::model::error::AppError;
use crate::model::users::User;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use salvo::Depot;
use std::sync::Arc;

//连接数据库
pub fn connect_database(depot: &mut Depot) -> PooledConnection<ConnectionManager<PgConnection>> {
    try_connect_database(depot).expect("数据库连接失败！")
}

pub fn try_connect_database(
    depot: &mut Depot,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, AppError> {
    let pool = depot
        .obtain::<Arc<DbPool>>()
        .map_err(|_| AppError::Internal("Database pool should be available in depot".to_string()))?;

    pool.get()
        .map_err(|e| AppError::Internal(format!("数据库连接失败: {}", e)))
}

pub fn current_user(depot: &mut Depot) -> Result<User, AppError> {
    depot.get::<User>("user")
        .cloned()
        .map_err(|_| AppError::UnAuthorized("未找到当前登录用户".to_string()))
}
