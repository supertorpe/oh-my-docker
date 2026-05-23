use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::event::AppEvent;

fn repo() -> &'static str {
    let url = env!("CARGO_PKG_REPOSITORY");
    url.strip_prefix("https://github.com/").unwrap_or(url)
}

fn archive_ext() -> &'static str {
    #[cfg(target_os = "windows")]
    { "zip" }
    #[cfg(not(target_os = "windows"))]
    { "tar.gz" }
}

fn target_triple() -> &'static str {
    #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
    { "x86_64-unknown-linux-gnu" }
    #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "musl"))]
    { "x86_64-unknown-linux-musl" }
    #[cfg(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))]
    { "aarch64-unknown-linux-gnu" }
    #[cfg(all(target_arch = "aarch64", target_os = "linux", target_env = "musl"))]
    { "aarch64-unknown-linux-musl" }
    #[cfg(all(target_arch = "arm", target_os = "linux", target_env = "gnu"))]
    { "armv7-unknown-linux-gnueabihf" }
    #[cfg(all(target_arch = "x86", target_os = "linux", target_env = "gnu"))]
    { "i686-unknown-linux-gnu" }
    #[cfg(all(target_arch = "x86", target_os = "linux", target_env = "musl"))]
    { "i686-unknown-linux-musl" }
    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    { "x86_64-apple-darwin" }
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    { "aarch64-apple-darwin" }
    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    { "x86_64-pc-windows-msvc" }
    #[cfg(all(target_arch = "x86", target_os = "windows"))]
    { "i686-pc-windows-msvc" }
    #[cfg(not(any(
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
        all(target_arch = "x86_64", target_os = "linux", target_env = "musl"),
        all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"),
        all(target_arch = "aarch64", target_os = "linux", target_env = "musl"),
        all(target_arch = "arm", target_os = "linux", target_env = "gnu"),
        all(target_arch = "x86", target_os = "linux", target_env = "gnu"),
        all(target_arch = "x86", target_os = "linux", target_env = "musl"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "aarch64", target_os = "macos"),
        all(target_arch = "x86_64", target_os = "windows"),
        all(target_arch = "x86", target_os = "windows"),
    )))]
    { "unknown" }
}

fn check_for_update(current_version: &str) -> Result<Option<(String, String)>> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo());
    let resp = ureq::get(&url)
        .header("User-Agent", "omdocker")
        .header("Accept", "application/json")
        .call()?;

    if resp.status() == 204 || resp.status() == 404 {
        return Ok(None);
    }

    let json: serde_json::Value = resp.into_body().read_json()?;
    let latest_tag = json["tag_name"].as_str().unwrap_or("v0.0.0");
    let latest_version = latest_tag.strip_prefix('v').unwrap_or(latest_tag);

    if latest_version <= current_version {
        return Ok(None);
    }

    let target = target_triple();
    let archive_name = format!("omdocker_{}_{}.{}", latest_version, target, archive_ext());
    if let Some(assets) = json["assets"].as_array() {
        for asset in assets {
            let name = asset["name"].as_str().unwrap_or("");
            if name == archive_name {
                if let Some(url) = asset["browser_download_url"].as_str() {
                    return Ok(Some((latest_version.to_string(), url.to_string())));
                }
            }
        }
    }

    Ok(None)
}

fn perform_update(version: &str, url: &str) -> Result<()> {
    let current_exe = std::env::current_exe()?;
    let temp_dir = std::env::temp_dir().join(format!("omdocker_update_{}", version));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)?;
    let archive = temp_dir.join("archive.tar.gz");

    let resp = ureq::get(url)
        .header("User-Agent", "omdocker")
        .call()?;
    let data = resp.into_body().read_to_vec()?;
    std::fs::write(&archive, &data)?;

    let status = if url.ends_with(".zip") {
        std::process::Command::new("unzip")
            .args([&archive.to_string_lossy(), "-d", &temp_dir.to_string_lossy()])
            .status()?
    } else {
        std::process::Command::new("tar")
            .args(["-xzf", &archive.to_string_lossy(), "-C", &temp_dir.to_string_lossy()])
            .status()?
    };
    if !status.success() {
        return Err(anyhow::anyhow!("archive extraction failed"));
    }

    let binary = find_binary(&temp_dir)
        .ok_or_else(|| anyhow::anyhow!("omdocker binary not found in archive"))?;

    std::fs::rename(&binary, &current_exe)?;
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

fn find_binary(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_binary(&path) {
                return Some(found);
            }
        } else if path.file_name().is_some_and(|n| n == "omdocker" || n == "omdocker.exe") {
            return Some(path);
        }
    }
    None
}

pub fn spawn_check_update(tx: UnboundedSender<AppEvent>) {
    let current = env!("CARGO_PKG_VERSION").to_string();
    tokio::task::spawn_blocking(move || {
        match check_for_update(&current) {
            Ok(Some((version, url))) => {
                tx.send(AppEvent::UpdateAvailable(version, url)).ok();
            }
            Ok(None) => {
                tx.send(AppEvent::UpdateNotAvailable).ok();
            }
            Err(e) => {
                tx.send(AppEvent::Error(format!("Update check: {}", e))).ok();
            }
        }
    });
}

pub fn spawn_download_update(tx: UnboundedSender<AppEvent>, version: String, download_url: String) {
    tokio::task::spawn_blocking(move || {
        match perform_update(&version, &download_url) {
            Ok(()) => {
                tx.send(AppEvent::Error(format!("Updated to v{}! Restart to apply.", version))).ok();
            }
            Err(e) => {
                tx.send(AppEvent::Error(format!("Update failed: {}", e))).ok();
            }
        }
    });
}
