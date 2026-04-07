use anyhow::{Context, Result, anyhow};
use apk_info::Apk;
use std::fs;
use std::path::Path;
use uuid::Uuid;

const IMAGE_EXTENSIONS: &[&str] = &["png", "webp", "jpg", "jpeg"];
const DENSITY_ORDER: &[&str] = &[
    "xxxhdpi", "xxhdpi", "xhdpi", "hdpi", "mdpi", "ldpi", "anydpi", "nodpi",
];

#[derive(Debug, Clone)]
pub struct ApkMetadata {
    pub file_name: String,
    pub app_name: String,
    pub package_name: String,
    pub app_icon_path: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<String>,
    pub file_size: u64,
}

pub fn extract_apk_metadata(
    apk_path: &Path,
    file_name: &str,
    upload_dir: &Path,
) -> Result<ApkMetadata> {
    let apk = Apk::new(apk_path).context("解析 APK 文件失败")?;
    let file_size = fs::metadata(apk_path)
        .with_context(|| format!("读取文件大小失败: {}", apk_path.display()))?
        .len();

    let package_name = apk
        .get_package_name()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("APK 缺少有效的包名"))?;

    let app_name = apk
        .get_application_label()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| fallback_app_name(apk_path, file_name));

    let icon_resource = apk
        .get_application_icon()
        .or_else(|| apk.get_attribute_value("application", "roundIcon"));
    let app_icon_path = save_icon_from_apk(&apk, icon_resource.as_deref(), upload_dir)
        .context("提取 APP 图标失败")?;

    Ok(ApkMetadata {
        file_name: file_name.to_string(),
        app_name,
        package_name,
        app_icon_path,
        version_name: apk.get_version_name(),
        version_code: apk.get_version_code(),
        file_size,
    })
}

fn fallback_app_name(apk_path: &Path, apk_name: &str) -> String {
    Path::new(apk_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            apk_path
                .file_stem()
                .and_then(|value| value.to_str())
                .filter(|value| !value.trim().is_empty())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "unknown_app".to_string())
}

fn save_icon_from_apk(
    apk: &Apk,
    icon_resource: Option<&str>,
    upload_dir: &Path,
) -> Result<Option<String>> {
    let Some(resource) = icon_resource else {
        return Ok(None);
    };

    let icon_entry = resolve_icon_entry(apk, resource);
    let Some(icon_entry) = icon_entry else {
        return Ok(None);
    };

    let (bytes, _) = apk
        .read(&icon_entry)
        .with_context(|| format!("读取 APK 图标资源失败: {icon_entry}"))?;

    if bytes.is_empty() {
        return Ok(None);
    }

    let extension = Path::new(&icon_entry)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .filter(|value| IMAGE_EXTENSIONS.contains(&value.as_str()))
        .unwrap_or_else(|| "png".to_string());

    let icon_dir = upload_dir.join("icons");
    fs::create_dir_all(&icon_dir)
        .with_context(|| format!("创建图标目录失败: {}", icon_dir.display()))?;

    let icon_name = format!("{}.{}", Uuid::new_v4(), extension);
    let icon_path = icon_dir.join(icon_name);
    fs::write(&icon_path, bytes)
        .with_context(|| format!("写入图标文件失败: {}", icon_path.display()))?;

    Ok(Some(normalize_relative_path(&icon_path)))
}

fn resolve_icon_entry(apk: &Apk, resource: &str) -> Option<String> {
    let normalized = normalize_apk_entry(resource);
    if is_image_entry(&normalized) {
        return Some(normalized);
    }

    if normalized.ends_with(".xml") {
        return fallback_image_entry(apk, &normalized);
    }

    fallback_image_entry(apk, &normalized)
}

fn normalize_apk_entry(resource: &str) -> String {
    resource.trim_start_matches('/').replace('\\', "/")
}

fn is_image_entry(resource: &str) -> bool {
    Path::new(resource)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .is_some_and(|value| IMAGE_EXTENSIONS.contains(&value.as_str()))
}

fn fallback_image_entry(apk: &Apk, resource: &str) -> Option<String> {
    let resource_path = Path::new(resource);
    let stem = resource_path.file_stem()?.to_str()?;

    let mut best_match: Option<(usize, String)> = None;
    for entry in apk.namelist() {
        let normalized = normalize_apk_entry(entry);
        if !normalized.starts_with("res/") || !is_image_entry(&normalized) {
            continue;
        }

        let entry_path = Path::new(&normalized);
        let entry_stem = match entry_path.file_stem().and_then(|value| value.to_str()) {
            Some(value) => value,
            None => continue,
        };

        if entry_stem != stem {
            continue;
        }

        let Some(parent) = entry_path.parent().and_then(|value| value.to_str()) else {
            continue;
        };

        let priority = density_priority(parent);
        match &best_match {
            Some((current_priority, _)) if priority >= *current_priority => {}
            _ => best_match = Some((priority, normalized)),
        }
    }

    best_match.map(|(_, value)| value)
}

fn density_priority(parent: &str) -> usize {
    DENSITY_ORDER
        .iter()
        .position(|density| parent.contains(density))
        .unwrap_or(DENSITY_ORDER.len())
}

fn normalize_relative_path(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    let cwd = std::env::current_dir()
        .ok()
        .map(|dir| dir.to_string_lossy().replace('\\', "/"));

    match cwd {
        Some(cwd) if path.starts_with(&cwd) => {
            path[cwd.len()..].trim_start_matches('/').to_string()
        }
        _ => path,
    }
}
