use tracing::info;

mod logging;
mod middleware;
mod server;

pub mod api;
pub mod db;
pub mod model;
pub mod schema;
pub mod store;
pub mod utils;

#[tokio::main]
async fn main() {
    // 兜底：无论 tracing 是否初始化成功，都确保容器 stdout 有启动信息
    println!("AppUpdateService starting...");

    // 必须把 guard 存活到进程结束，否则日志线程会停，文件可能写不出来
    let _guard = match logging::init::init_logging("logs", 14) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("fatal: init_logging failed: {e}");
            return;
        }
    }; // 保留 14 天

    info!("app starting");
    server::run().await;
    eprintln!("fatal: server::run() returned; exiting");
}
