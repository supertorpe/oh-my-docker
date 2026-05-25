use bollard::Docker;
use anyhow::Result;

use crate::app::event::VolumeEntry;

pub async fn remove_volume(docker: &Docker, name: &str) -> Result<()> {
    docker.remove_volume(name, None::<bollard::volume::RemoveVolumeOptions>).await.map_err(Into::into)
}

pub async fn list_volumes(docker: &Docker) -> Result<Vec<VolumeEntry>> {
    let response = docker.list_volumes::<String>(None::<bollard::volume::ListVolumesOptions<String>>).await?;

    let entries = response
        .volumes
        .unwrap_or_default()
        .into_iter()
        .map(|v| VolumeEntry {
            name: v.name,
            driver: v.driver,
            mountpoint: v.mountpoint,
        })
        .collect();

    Ok(entries)
}