use std::env;
use std::time::{Duration, Instant};

use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use tracing::{info, warn};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection_pool() -> DbPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool");

    // 容器部署时，DB 可能尚未 ready；这里做启动期重试，避免进程直接退出导致容器重启
    // - `DB_CONNECT_MAX_WAIT_SECS`：最大等待秒数；默认 0 表示无限等待
    // - `DB_CONNECT_RETRY_DELAY_MS`：每次重试间隔；默认 1000ms
    let max_wait_secs = env::var("DB_CONNECT_MAX_WAIT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let retry_delay_ms = env::var("DB_CONNECT_RETRY_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1000);

    let deadline = if max_wait_secs == 0 {
        None
    } else {
        Some(Instant::now() + Duration::from_secs(max_wait_secs))
    };

    loop {
        match pool.get() {
            Ok(conn) => {
                drop(conn);
                info!("database connection established");
                break;
            }
            Err(e) => {
                if deadline.is_some_and(|d| Instant::now() >= d) {
                    panic!("database connection not available after {max_wait_secs}s: {e}");
                }
                warn!(error = %e, "database not ready, retrying...");
                std::thread::sleep(Duration::from_millis(retry_delay_ms.max(200)));
            }
        }
    }

    pool
}
