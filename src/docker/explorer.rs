use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Result;
use bollard::container::{
    Config, CreateContainerOptions, DownloadFromContainerOptions, ListContainersOptions,
    LogOutput, StartContainerOptions, StopContainerOptions,
    UploadToContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::HostConfig;
use bollard::Docker;
use futures_util::StreamExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::event::AppEvent;
use crate::app::state::ExplorerEntry;

fn format_mode(mode: u32) -> String {
    let mut s = String::with_capacity(10);
    s.push(if mode & 0o040000 != 0 { 'd' } else { '-' });
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    s
}

fn parse_ls_line(line: &str) -> Option<ExplorerEntry> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("total ") {
        return None;
    }
    let perms_end = if line.as_bytes().get(0).copied() == Some(b'd')
        || line.as_bytes().get(0).copied() == Some(b'-')
        || line.as_bytes().get(0).copied() == Some(b'l')
    {
        let mut idx = 0;
        let bytes = line.as_bytes();
        // Find the end of the first whitespace-delimited field
        while idx < bytes.len() && bytes[idx] != b' ' {
            idx += 1;
        }
        idx
    } else {
        return None;
    };

    let perms = &line[..perms_end];
    let is_dir = perms.starts_with('d');
    let rest = line[perms_end..].trim_start();

    // rest: links owner group size month day time name
    let fields: Vec<&str> = rest.split_whitespace().collect();
    if fields.len() < 8 {
        return None;
    }

    let size: i64 = fields[3].parse().unwrap_or(0);
    let month = fields[4];
    let day = fields[5];
    let time_or_year = fields[6];
    let modified = format!("{} {} {}", month, day, time_or_year);

    // fields 7+ is the filename (may contain spaces)
    let mut name = fields[7..].join(" ");
    // Remove trailing symlink arrow
    if let Some(pos) = name.find(" -> ") {
        name.truncate(pos);
    }
    let name = name.strip_suffix('/').unwrap_or(&name).to_string();

    if name == "." || name == ".." {
        return None;
    }

    Some(ExplorerEntry {
        name,
        is_dir,
        size,
        modified,
        permissions: perms.to_string(),
    })
}

pub async fn list_container_dir(
    docker: &Docker,
    container_id: &str,
    path: &str,
) -> Result<Vec<ExplorerEntry>> {
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions {
                cmd: Some(vec![
                    "ls".to_string(),
                    "-lap".to_string(),
                    "--".to_string(),
                    path.to_string(),
                ]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await?;

    let output = docker.start_exec(&exec.id, None::<StartExecOptions>).await?;

    let mut stdout_bytes = Vec::new();

    if let StartExecResults::Attached { mut output, .. } = output {
        while let Some(msg_result) = output.next().await {
            match msg_result {
                Ok(LogOutput::StdOut { message }) => {
                    stdout_bytes.extend_from_slice(&message);
                }
                Ok(LogOutput::StdErr { message }) => {
                    let stderr = String::from_utf8_lossy(&message);
                    if !stderr.trim().is_empty() {
                        // ignore stderr, stdout may still have partial output
                    }
                }
                _ => {}
            }
        }
    }

    let stdout = String::from_utf8_lossy(&stdout_bytes);
    let mut entries: Vec<ExplorerEntry> = stdout.lines().filter_map(parse_ls_line).collect();

    entries.sort_by(|a, b| {
        a.is_dir.cmp(&b.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

pub async fn list_host_dir(path: &str) -> Result<Vec<ExplorerEntry>> {
    let mut entries = Vec::new();
    let dir = std::fs::read_dir(path)?;

    for entry in dir {
        let entry = entry?;
        let name = entry.file_name()
            .to_string_lossy()
            .to_string();

        if name == "." || name == ".." {
            continue;
        }

        let meta = entry.metadata().ok();
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = meta.as_ref().map(|m| m.len() as i64).unwrap_or(0);
        let modified = meta.as_ref()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dt: chrono::DateTime<chrono::Local> = t.into();
                dt.format("%b %e %H:%M").to_string()
            })
            .unwrap_or_default();
        let permissions = meta.as_ref()
            .map(|m| format_mode(m.permissions().mode() & 0o7777))
            .unwrap_or_default();

        entries.push(ExplorerEntry {
            name,
            is_dir,
            size,
            modified,
            permissions,
        });
    }

    entries.sort_by(|a, b| {
        a.is_dir.cmp(&b.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

const VOLUME_HELPER_IMAGE: &str = "alpine:latest";
const VOLUME_MOUNT_PATH: &str = "/volume_data";

fn volume_container_name(volume_name: &str) -> String {
    format!("omdocker-vol-{}", volume_name)
}

fn volume_container_path(path: &str) -> String {
    if path == "/" {
        VOLUME_MOUNT_PATH.to_string()
    } else {
        format!("{}{}", VOLUME_MOUNT_PATH, path.trim_end_matches('/'))
    }
}

/// Ensure a helper container exists for the given Docker volume, start it if
/// stopped, and return its ID. The container mounts the volume and runs `sleep`
/// for a very long time so it can be reused across directory listings.
pub async fn ensure_volume_helper(docker: &Docker, volume_name: &str) -> Result<String> {
    let name = volume_container_name(volume_name);

    // Check if the container already exists (running or stopped)
    let existing = docker
        .list_containers::<&str>(Some(ListContainersOptions {
            all: true,
            filters: HashMap::from([("name", vec![name.as_str()])]),
            ..Default::default()
        }))
        .await?
        .into_iter()
        .next();

    if let Some(c) = existing {
        if let Some(ref id) = c.id {
            if c.state != Some("running".to_string()) {
                docker.start_container(id, None::<StartContainerOptions<&str>>).await?;
            }
            return Ok(id.clone());
        }
    }

    // Create a new helper container with auto-remove (--rm)
    let container = docker
        .create_container(
            Some(CreateContainerOptions {
                name: name.as_str(),
                platform: None,
            }),
            Config {
                image: Some(VOLUME_HELPER_IMAGE),
                cmd: Some(vec!["sleep".into(), "86400".into()]),
                host_config: Some(HostConfig {
                    binds: Some(vec![format!("{}:{}:z", volume_name, VOLUME_MOUNT_PATH)]),
                    auto_remove: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    docker
        .start_container(&container.id, None::<StartContainerOptions<&str>>)
        .await?;

    Ok(container.id)
}

/// Stop the helper container for a given Docker volume.
/// With auto_remove enabled, Docker automatically removes it on stop.
pub async fn remove_volume_helper(docker: &Docker, volume_name: &str) -> Result<()> {
    let name = volume_container_name(volume_name);
    docker
        .stop_container(&name, None::<StopContainerOptions>)
        .await
        .or_else(|e| {
            // If container already gone, that's fine
            if e.to_string().contains("404") || e.to_string().contains("not found") {
                Ok(())
            } else {
                Err(e)
            }
        })?;
    Ok(())
}

pub async fn list_volume_dir(
    docker: &Docker,
    volume_name: &str,
    path: &str,
) -> Result<Vec<ExplorerEntry>> {
    let container_id = ensure_volume_helper(docker, volume_name).await?;
    let container_path = volume_container_path(path);
    list_container_dir(docker, &container_id, &container_path).await
}

pub async fn copy_to_container(
    docker: &Docker,
    container_id: &str,
    host_path: String,
    container_path: String,
) -> Result<()> {
    let path = Path::new(&host_path);
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", host_path));
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);

        if path.is_dir() {
            builder.append_dir_all(file_name, path)?;
        } else {
            let mut file = std::fs::File::open(path)?;
            builder.append_file(file_name, &mut file)?;
        }

        builder.finish()?;
    }

    docker
        .upload_to_container(
            container_id,
            Some(UploadToContainerOptions {
                path: container_path,
                no_overwrite_dir_non_dir: "false".to_string(),
            }),
            tar_bytes.into(),
        )
        .await?;

    Ok(())
}

pub async fn copy_from_container(
    docker: &Docker,
    container_id: &str,
    container_path: String,
    host_dest_dir: &str,
) -> Result<std::path::PathBuf> {
    let stream = docker.download_from_container(
        container_id,
        Some(DownloadFromContainerOptions {
            path: container_path.clone(),
        }),
    );

    let mut all_bytes = Vec::new();
    let mut stream = stream;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        all_bytes.extend_from_slice(&chunk);
    }

    let dest_dir = Path::new(host_dest_dir);
    let mut archive = tar::Archive::new(all_bytes.as_slice());
    archive.unpack(dest_dir)?;

    let dest = dest_dir.join(
        Path::new(&container_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string()),
    );

    Ok(dest)
}

pub fn spawn_list_container_dir(
    docker: Docker,
    tx: UnboundedSender<AppEvent>,
    container_id: String,
    path: String,
) {
    tokio::spawn(async move {
        match list_container_dir(&docker, &container_id, &path).await {
            Ok(entries) => {
                let _ = tx.send(AppEvent::ExplorerContainerDirUpdated(container_id, path, entries));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::ExplorerTransferError(format!("Failed to list directory: {}", e)));
            }
        }
    });
}

pub async fn delete_in_container(
    docker: &Docker,
    container_id: &str,
    path: &str,
) -> Result<()> {
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions {
                cmd: Some(vec![
                    "rm".to_string(),
                    "-rf".to_string(),
                    "--".to_string(),
                    path.to_string(),
                ]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await?;

    let output = docker.start_exec(&exec.id, None::<StartExecOptions>).await?;

    if let StartExecResults::Attached { mut output, .. } = output {
        while let Some(msg_result) = output.next().await {
            match msg_result {
                Ok(LogOutput::StdErr { message }) => {
                    let stderr = String::from_utf8_lossy(&message);
                    let trimmed = stderr.trim();
                    if !trimmed.is_empty() {
                        return Err(anyhow::anyhow!("{}", trimmed));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub async fn create_in_container(
    docker: &Docker,
    container_id: &str,
    path: &str,
    is_dir: bool,
) -> Result<()> {
    let cmd = if is_dir {
        vec!["mkdir".to_string(), "-p".to_string(), "--".to_string(), path.to_string()]
    } else {
        vec!["touch".to_string(), "--".to_string(), path.to_string()]
    };
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions {
                cmd: Some(cmd),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await?;

    let output = docker.start_exec(&exec.id, None::<StartExecOptions>).await?;

    if let StartExecResults::Attached { mut output, .. } = output {
        while let Some(msg_result) = output.next().await {
            match msg_result {
                Ok(LogOutput::StdErr { message }) => {
                    let stderr = String::from_utf8_lossy(&message);
                    let trimmed = stderr.trim();
                    if !trimmed.is_empty() {
                        return Err(anyhow::anyhow!("{}", trimmed));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub async fn rename_in_container(
    docker: &Docker,
    container_id: &str,
    old_path: &str,
    new_path: &str,
) -> Result<()> {
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions {
                cmd: Some(vec![
                    "mv".to_string(),
                    "--".to_string(),
                    old_path.to_string(),
                    new_path.to_string(),
                ]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await?;

    let output = docker.start_exec(&exec.id, None::<StartExecOptions>).await?;

    if let StartExecResults::Attached { mut output, .. } = output {
        while let Some(msg_result) = output.next().await {
            match msg_result {
                Ok(LogOutput::StdErr { message }) => {
                    let stderr = String::from_utf8_lossy(&message);
                    let trimmed = stderr.trim();
                    if !trimmed.is_empty() {
                        return Err(anyhow::anyhow!("{}", trimmed));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
