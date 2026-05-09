use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tracing_subscriber::filter;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

pub struct LoggingGuard {
    _app_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
    _access_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

pub fn init_logging(logs_dir: impl Into<PathBuf>, retention_days: u64) -> Result<LoggingGuard> {
    let logs_dir = logs_dir.into();

    // 控制台（容器/开发都需要）：输出到 stdout，方便 `docker logs` 排障
    let console_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let console_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_filter(console_filter);

    // 文件日志：失败时退化为仅控制台日志（避免容器无任何输出，难以排障）
    let mut app_guard = None;
    let mut access_guard = None;
    let (app_layer, access_layer) = match std::fs::create_dir_all(&logs_dir) {
        Ok(()) => {
            // 按天滚动
            let app_file = tracing_appender::rolling::daily(&logs_dir, "app.log");
            let access_file = tracing_appender::rolling::daily(&logs_dir, "access.log");

            let (app_nb, g1) = tracing_appender::non_blocking::NonBlockingBuilder::default()
                .buffered_lines_limit(50_000)
                .lossy(true)
                .finish(app_file);
            let (access_nb, g2) = tracing_appender::non_blocking::NonBlockingBuilder::default()
                .buffered_lines_limit(50_000)
                .lossy(true)
                .finish(access_file);
            app_guard = Some(g1);
            access_guard = Some(g2);

            // 业务日志（排除 access）
            let app_layer = tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_target(true)
                .with_level(true)
                .with_writer(app_nb)
                .with_filter(filter::filter_fn(|meta| meta.target() != "access"));

            // access 日志（只写必要字段，减少 IO）
            let access_layer = tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_target(false)
                .with_level(false)
                .with_writer(access_nb)
                .with_filter(filter::filter_fn(|meta| meta.target() == "access"));

            (Some(app_layer), Some(access_layer))
        }
        Err(e) => {
            eprintln!(
                "warn: failed to create logs dir {:?}: {e}; falling back to console-only logging",
                logs_dir
            );
            (None, None)
        }
    };

    let subscriber = tracing_subscriber::registry().with(console_layer);
    if let (Some(app_layer), Some(access_layer)) = (app_layer, access_layer) {
        subscriber.with(app_layer).with(access_layer).init();
        spawn_retention_cleanup(logs_dir.clone(), retention_days);
    } else {
        subscriber.init();
    }

    Ok(LoggingGuard {
        _app_guard: app_guard,
        _access_guard: access_guard,
    })
}

fn spawn_retention_cleanup(logs_dir: PathBuf, retention_days: u64) {
    let interval = Duration::from_secs(6 * 3600);

    // 优先用 tokio runtime；没有就退化到 std::thread，避免 panic
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(async move {
            loop {
                if let Err(e) = cleanup_old_logs(&logs_dir, retention_days) {
                    tracing::warn!(error = ?e, "failed to cleanup old log files");
                }
                tokio::time::sleep(interval).await;
            }
        });
    } else {
        std::thread::spawn(move || {
            loop {
                if let Err(e) = cleanup_old_logs(&logs_dir, retention_days) {
                    tracing::warn!(error = ?e, "failed to cleanup old log files");
                }
                std::thread::sleep(interval);
            }
        });
    }
}

fn cleanup_old_logs(dir: &Path, retention_days: u64) -> std::io::Result<()> {
    let now = std::time::SystemTime::now();
    let keep = Duration::from_secs(retention_days.saturating_mul(24 * 3600));

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
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
