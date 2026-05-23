use bollard::Docker;
use anyhow::Result;
use futures_util::StreamExt;

use crate::app::event::StatEntry;

pub async fn list_statistics(docker: &Docker) -> Result<Vec<StatEntry>> {
    use bollard::container::StatsOptions;

    let containers = docker.list_containers::<String>(Some(bollard::container::ListContainersOptions {
        all: false,
        ..Default::default()
    }))
    .await?;

    let mut entries = Vec::new();
    for container in containers {
        let id = container.id.unwrap_or_default();
        let name = container
            .names
            .unwrap_or_default()
            .first()
            .cloned()
            .unwrap_or_default()
            .trim_start_matches('/')
            .to_string();

        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };
        let mut stream = docker.stats(&id, Some(options));
        if let Some(Ok(stats)) = stream.next().await {
            let cpu_delta = stats.cpu_stats.cpu_usage.total_usage.saturating_sub(
                stats.precpu_stats.cpu_usage.total_usage,
            );
            let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0).saturating_sub(
                stats.precpu_stats.system_cpu_usage.unwrap_or(0),
            );
            let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
            let cpu_percent = if system_delta > 0 && cpu_delta > 0 {
                (cpu_delta as f64 / system_delta as f64) * num_cpus * 100.0
            } else {
                0.0
            };

            let mem = &stats.memory_stats;
            let memory_usage = mem.usage.unwrap_or(0);
            let memory_limit = mem.limit.unwrap_or(0);
            let memory_percent = if memory_limit > 0 {
                (memory_usage as f64 / memory_limit as f64) * 100.0
            } else {
                0.0
            };

            let mut net_rx = 0u64;
            let mut net_tx = 0u64;
            if let Some(networks) = &stats.networks {
                for net in networks.values() {
                    net_rx = net_rx.saturating_add(net.rx_bytes);
                    net_tx = net_tx.saturating_add(net.tx_bytes);
                }
            }

            let mut block_read = 0u64;
            let mut block_write = 0u64;
            if let Some(ref io_serviced) = stats.blkio_stats.io_service_bytes_recursive {
                for entry in io_serviced {
                    match entry.op.as_str() {
                        "read" => block_read = block_read.saturating_add(entry.value),
                        "write" => block_write = block_write.saturating_add(entry.value),
                        _ => {}
                    }
                }
            }

            let pids = stats
                .pids_stats
                .current
                .unwrap_or(0);

            entries.push(StatEntry {
                container_id: id,
                name,
                cpu_percent,
                memory_usage,
                memory_limit,
                memory_percent,
                net_rx,
                net_tx,
                block_read,
                block_write,
                pids,
            });
        }
    }

    Ok(entries)
}
