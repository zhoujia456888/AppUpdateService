use crate::db::DbPool;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use salvo::Depot;
use std::sync::Arc;

//连接数据库
pub fn connect_database(depot: &mut Depot) -> PooledConnection<ConnectionManager<PgConnection>> {
    //连接数据库
    let pool = depot
        .obtain::<Arc<DbPool>>()
        .expect("Database pool should be available in depot");
    let conn = pool.get().expect("数据库连接失败！");
    conn
}
