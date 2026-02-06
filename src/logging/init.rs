use anyhow::Result;
use std::{path::PathBuf, time::Duration};
use tracing_subscriber::filter;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

pub struct LoggingGuard {
    _app_guard: tracing_appender::non_blocking::WorkerGuard,
    _access_guard: tracing_appender::non_blocking::WorkerGuard,
}

pub fn init_logging(logs_dir: impl Into<PathBuf>, retention_days: u64) -> Result<LoggingGuard> {
    let logs_dir = logs_dir.into();
    std::fs::create_dir_all(&logs_dir)?;

    // 文件滚动（按天）
    let app_file = tracing_appender::rolling::daily(&logs_dir, "app.log");
    let access_file = tracing_appender::rolling::daily(&logs_dir, "access.log");

    // 异步写入（高并发）
    let (app_nb, app_guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .buffered_lines_limit(50_000)
        .lossy(true) // 极端高并发下允许丢日志，吞吐最高
        .finish(app_file);

    let (access_nb, access_guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .buffered_lines_limit(50_000)
        .lossy(true)
        .finish(access_file);

    // 业务日志（排除 access）
    let app_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_writer(app_nb)
        .with_filter(filter::filter_fn(|meta| meta.target() != "access"));

    let access_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(false)
        .with_level(false)
        .with_writer(access_nb)
        .with_filter(filter::filter_fn(|meta| meta.target() == "access"));


    // 控制台（开发用）
    let console_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    let console_layer = tracing_subscriber::fmt::layer().with_filter(console_filter);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(app_layer)
        .with(access_layer)
        .init();

    spawn_retention_cleanup(logs_dir, retention_days);

    Ok(LoggingGuard {
        _app_guard: app_guard,
        _access_guard: access_guard,
    })
}

fn spawn_retention_cleanup(logs_dir: PathBuf, retention_days: u64) {
    // 每 6 小时执行一次
    let interval = Duration::from_secs(6 * 3600);

    tokio::spawn(async move {
        loop {
            if let Err(e) = cleanup_old_logs(&logs_dir, retention_days) {
                tracing::warn!(
                    error = ?e,
                    "failed to cleanup old log files"
                );
            }
            tokio::time::sleep(interval).await;
        }
    });
}

fn cleanup_old_logs(dir: &PathBuf, retention_days: u64) -> std::io::Result<()> {
    let now = std::time::SystemTime::now();
    let keep = Duration::from_secs(retention_days * 24 * 3600);

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        // 只清理我们生成的日志文件
        if !(name.starts_with("app.log") || name.starts_with("access.log")) {
            continue;
        }

        let meta = entry.metadata()?;
        let modified = meta.modified().unwrap_or(now);

        if now.duration_since(modified).unwrap_or(Duration::ZERO) > keep {
            let _ = std::fs::remove_file(&path);
        }
    }

    Ok(())
}
