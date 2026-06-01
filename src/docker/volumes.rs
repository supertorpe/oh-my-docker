use bollard::Docker;
use anyhow::Result;

use crate::app::event::VolumeEntry;

pub async fn remove_volume(docker: &Docker, name: &str) -> Result<()> {
    docker.remove_volume(name, None::<bollard::volume::RemoveVolumeOptions>).await.map_err(Into::into)
}

pub async fn list_volumes(docker: &Docker) -> Result<Vec<VolumeEntry>> {
    let response = docker.list_volumes::<String>(None::<bollard::volume::ListVolumesOptions<String>>).await?;

    let size_map = docker.df().await
        .ok()
        .and_then(|d| d.volumes)
        .map(|vols| {
            vols.into_iter()
                .filter_map(|v| {
                    let name = v.name.clone();
                    v.usage_data.map(|u| (name, u.size))
                })
                .collect::<std::collections::HashMap<_, _>>()
        })
        .unwrap_or_default();

    let entries = response
        .volumes
        .unwrap_or_default()
        .into_iter()
        .map(|v| VolumeEntry {
            name: v.name.clone(),
            driver: v.driver,
            mountpoint: v.mountpoint,
            size: *size_map.get(&v.name).unwrap_or(&0),
        })
        .collect();

    Ok(entries)
}