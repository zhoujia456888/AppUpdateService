use crate::db::DbPool;
use crate::schema::app_manage;
use diesel::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

const APP_MANAGE_DIR: &str = "app_manage";
const APK_DIR: &str = "apk";
const ICON_DIR: &str = "icons";
const ICON_PUBLIC_ROUTE: &str = "icon";
const PUBLIC_APP_MANAGE_PREFIX: &str = "/api/public/app_manage";

pub fn start_app_manage_cleanup_task(pool: Arc<DbPool>) {
    thread::spawn(move || {
        run_cleanup_once(&pool);

        loop {
            let sleep_duration = duration_until_next_run();
            thread::sleep(sleep_duration);
            run_cleanup_once(&pool);
        }
    });
}

fn duration_until_next_run() -> Duration {
    let now = chrono::Local::now();
    let next_run = (now + chrono::Duration::days(1))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("生成下一次清理时间失败");

    let seconds = (next_run - now.naive_local()).num_seconds().max(60);
    Duration::from_secs(seconds as u64)
}

fn run_cleanup_once(pool: &Arc<DbPool>) {
    match cleanup_unused_files(pool) {
        Ok((apk_count, icon_count)) => {
            info!(
                apk_deleted = apk_count,
                icon_deleted = icon_count,
                "app_manage 无效文件清理完成"
            );
        }
        Err(e) => {
            error!(error = %e, "app_manage 无效文件清理失败");
        }
    }
}

fn cleanup_unused_files(pool: &Arc<DbPool>) -> anyhow::Result<(usize, usize)> {
    let mut conn = pool.get()?;
    let referenced_files = app_manage::table
        .select((app_manage::file_path, app_manage::app_icon_path))
        .filter(app_manage::is_delete.eq(false))
        .load::<(Option<String>, Option<String>)>(&mut conn)?;

    let mut apk_names = HashSet::new();
    let mut icon_names = HashSet::new();

    for (file_path, icon_path) in referenced_files {
        if let Some(name) = file_path
            .as_deref()
            .and_then(|value| extract_managed_filename(value, &[APK_DIR]))
        {
            apk_names.insert(name);
        }
        if let Some(name) = icon_path
            .as_deref()
            .and_then(|value| extract_managed_filename(value, &[ICON_DIR, ICON_PUBLIC_ROUTE]))
        {
            icon_names.insert(name);
        }
    }

    let apk_deleted = cleanup_directory(&PathBuf::from(APP_MANAGE_DIR).join(APK_DIR), &apk_names)?;
    let icon_deleted =
        cleanup_directory(&PathBuf::from(APP_MANAGE_DIR).join(ICON_DIR), &icon_names)?;

    Ok((apk_deleted, icon_deleted))
}

fn cleanup_directory(dir: &Path, referenced_names: &HashSet<String>) -> anyhow::Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }

    let mut deleted_count = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path
            .file_name()
            .and_then(|value| value.to_str())
            .map(|value| value.to_string())
        else {
            warn!(path = %path.display(), "跳过无法识别名称的文件");
            continue;
        };

        if referenced_names.contains(&file_name) {
            continue;
        }

        fs::remove_file(&path)?;
        deleted_count += 1;
        info!(path = %path.display(), "删除未引用文件");
    }

    Ok(deleted_count)
}

fn extract_managed_filename(value: &str, kinds: &[&str]) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    for kind in kinds {
        let public_prefix = format!("{PUBLIC_APP_MANAGE_PREFIX}/{kind}?name=");
        if let Some(name) = trimmed.strip_prefix(&public_prefix) {
            return normalize_filename(name);
        }
    }

    let normalized = trimmed.replace('\\', "/");
    if kinds.iter().any(|kind| {
        normalized.contains(&format!("/{kind}/"))
            || normalized.starts_with(&format!("{APP_MANAGE_DIR}/{kind}/"))
    }) {
        return normalized.rsplit('/').next().and_then(normalize_filename);
    }

    None
}

fn normalize_filename(name: &str) -> Option<String> {
    let filename = name
        .split('&')
        .next()
        .unwrap_or(name)
        .trim()
        .trim_start_matches('/');
    if filename.is_empty()
        || filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
    {
        return None;
    }
    Some(filename.to_string())
}
