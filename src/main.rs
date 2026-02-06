use tracing::info;

mod logging;
mod middleware;
mod server;

pub mod api;
pub mod db;
pub mod model;
pub mod schema;
pub mod utils;

#[tokio::main]
async fn main() {
    // 必须把 guard 存活到进程结束，否则日志线程会停，文件可能写不出来
    let _guard = logging::init::init_logging("logs", 14); // 保留 14 天

    info!("app starting");
    server::run().await;
}
