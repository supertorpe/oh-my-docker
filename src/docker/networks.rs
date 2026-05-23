use bollard::Docker;
use anyhow::Result;

use crate::app::event::NetworkEntry;

pub async fn remove_network(docker: &Docker, id: &str) -> Result<()> {
    docker.remove_network(id).await.map_err(Into::into)
}

pub async fn list_networks(docker: &Docker) -> Result<Vec<NetworkEntry>> {
    let networks = docker.list_networks::<String>(None::<bollard::network::ListNetworksOptions<String>>).await?;

    let entries = networks
        .into_iter()
        .map(|n| {
            let id = n.id.unwrap_or_default();
            let name = n.name.unwrap_or_default();
            let driver = n.driver.unwrap_or_default();
            let scope = n.scope.unwrap_or_default();
            let containers = n.containers.unwrap_or_default().len();

            let (subnet, gateway) = n
                .ipam
                .and_then(|ipam| {
                    ipam.config.and_then(|config| {
                        config.first().map(|c| {
                            (
                                c.subnet.clone().unwrap_or_default(),
                                c.gateway.clone().unwrap_or_default(),
                            )
                        })
                    })
                })
                .unwrap_or_default();

            NetworkEntry {
                id,
                name,
                driver,
                scope,
                subnet,
                gateway,
                containers,
            }
        })
        .collect();

    Ok(entries)
}