use bollard::Docker;
use bollard::network::InspectNetworkOptions;
use anyhow::Result;

use crate::app::event::NetworkEntry;

pub async fn remove_network(docker: &Docker, id: &str) -> Result<()> {
    docker.remove_network(id).await.map_err(Into::into)
}

pub async fn list_networks(docker: &Docker) -> Result<Vec<NetworkEntry>> {
    let network_list = docker.list_networks::<String>(None::<bollard::network::ListNetworksOptions<String>>).await?;

    let mut entries = Vec::with_capacity(network_list.len());

    for n in &network_list {
        let id = n.id.clone().unwrap_or_default();
        let name = n.name.clone().unwrap_or_default();
        let driver = n.driver.clone().unwrap_or_default();
        let scope = n.scope.clone().unwrap_or_default();

        let subnet = n.ipam.as_ref()
            .and_then(|ipam| ipam.config.as_ref())
            .and_then(|config| config.first())
            .and_then(|c| c.subnet.as_ref())
            .cloned()
            .unwrap_or_default();

        let gateway = n.ipam.as_ref()
            .and_then(|ipam| ipam.config.as_ref())
            .and_then(|config| config.first())
            .and_then(|c| c.gateway.as_ref())
            .cloned()
            .unwrap_or_default();

        let containers = if let Some(containers) = &n.containers {
            containers.len()
        } else {
            0
        };

        entries.push(NetworkEntry {
            id,
            name,
            driver,
            scope,
            subnet,
            gateway,
            containers,
        });
    }

    let network_ids: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();

    for network_id in &network_ids {
        if let Ok(inspected) = docker.inspect_network(network_id, Some(InspectNetworkOptions::<String>::default())).await {
            if let Some(ref mut entry) = entries.iter_mut().find(|e| e.id == *network_id) {
                if let Some(containers) = &inspected.containers {
                    entry.containers = containers.len();
                }
            }
        }
    }

    Ok(entries)
}