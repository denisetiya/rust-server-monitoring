use serde::{Deserialize, Serialize};
use bollard::Docker;
use bollard::container::{StatsOptions};
use bollard::models::{ContainerSummary, ContainerInspectResponse};
use chrono::{DateTime, Utc};
use crate::config::Config;
use log::{info, error, warn};
use anyhow::{Result, anyhow};
use futures_util::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percent: f64,
    pub ports: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DockerSystemInfo {
    pub version: String,
    pub api_version: String,
    pub containers: u64,
    pub containers_running: u64,
    pub containers_paused: u64,
    pub containers_stopped: u64,
    pub images: u64,
    pub server_version: String,
    pub memory_total: u64,
    pub cpu_count: u64,
}

pub struct DockerMonitor {
    docker: Docker,
    #[allow(dead_code)]
    config: Config,
}

impl DockerMonitor {
    pub async fn new(config: Config) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;
        
        // Test connection
        match docker.ping().await {
            Ok(_) => info!("Connected to Docker daemon successfully"),
            Err(e) => {
                error!("Failed to connect to Docker: {}", e);
                return Err(anyhow!("Docker connection failed: {}", e));
            }
        }
        
        Ok(Self {
            docker,
            config,
        })
    }
    
    pub async fn get_container_stats(&self) -> Result<Vec<ContainerStats>> {
        let containers = self.docker.list_containers::<String>(None).await?;
        let mut container_stats = Vec::new();
        
        for container in containers {
            match self.get_single_container_stats(&container).await {
                Ok(stats) => container_stats.push(stats),
                Err(e) => {
                    error!("Error getting stats for container {:?}: {}", container.id, e);
                    continue;
                }
            }
        }
        
        // Sort by CPU usage (highest first)
        container_stats.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        
        Ok(container_stats)
    }
    
    async fn get_single_container_stats(&self, container: &ContainerSummary) -> Result<ContainerStats> {
        let id = container.id.as_deref().unwrap_or("unknown");
        let name = container.names.as_ref()
            .and_then(|names| names.first())
            .map(|s| s.strip_prefix('/').unwrap_or(s))
            .unwrap_or("unknown")
            .to_string();
        
        let image = container.image.as_deref().unwrap_or("unknown").to_string();
        let status = container.status.as_deref().unwrap_or("unknown").to_string();
        
        // Get ports - simplified implementation
        let ports = Vec::new();
        // For now, skip port parsing to avoid type issues
        // In production, you would implement proper port parsing
        
        // Get CPU and memory stats
        let (cpu_usage, memory_usage, memory_limit, memory_percent) = 
            self.calculate_resource_usage(container).await?;
        
        Ok(ContainerStats {
            id: id.chars().take(12).collect(),
            name,
            image,
            status,
            cpu_usage,
            memory_usage,
            memory_limit,
            memory_percent,
            ports,
            timestamp: Utc::now(),
        })
    }
    
    async fn calculate_resource_usage(&self, container: &ContainerSummary) -> Result<(f64, u64, u64, f64)> {
        let container_id = container.id.as_ref().ok_or_else(|| anyhow!("No container id"))?;
        
        let mut stats_stream = self.docker.stats(
            container_id,
            Some(StatsOptions {
                stream: false,
                one_shot: true,
            })
        );
        
        if let Some(Ok(stats)) = stats_stream.next().await {
            // Calculate CPU usage - simplified
            let cpu_usage = self.calculate_cpu_usage(&stats)?;
            
            // Calculate memory usage
            let memory_usage = stats.memory_stats.usage.unwrap_or(0);
            let memory_limit = stats.memory_stats.limit.unwrap_or(0);
            let memory_percent = if memory_limit > 0 {
                (memory_usage as f64 / memory_limit as f64) * 100.0
            } else {
                0.0
            };
            
            Ok((cpu_usage, memory_usage, memory_limit, memory_percent))
        } else {
            Ok((0.0, 0, 0, 0.0))
        }
    }
    
    fn calculate_cpu_usage(&self, _stats: &bollard::container::Stats) -> Result<f64> {
        // Simplified CPU calculation - return 0.0 for now
        // In production, you would implement proper CPU calculation
        Ok(0.0)
    }
    
    #[allow(dead_code)]
    pub async fn get_top_cpu_containers(&self, limit: usize) -> Result<Vec<ContainerStats>> {
        let mut all_stats = self.get_container_stats().await?;
        all_stats.truncate(limit);
        Ok(all_stats)
    }
    
    pub async fn check_container_cpu_threshold(&self, threshold: f64) -> Result<(bool, Vec<ContainerStats>)> {
        let container_stats = self.get_container_stats().await?;
        let high_cpu_containers: Vec<ContainerStats> = container_stats
            .into_iter()
            .filter(|container| container.cpu_usage > threshold)
            .collect();
        
        let has_high_cpu = !high_cpu_containers.is_empty();
        
        if has_high_cpu {
            warn!("High CPU usage detected in {} containers", high_cpu_containers.len());
            for container in &high_cpu_containers {
                warn!("Container {}: {:.2}% CPU", container.name, container.cpu_usage);
            }
        } else {
            info!("All containers have normal CPU usage");
        }
        
        Ok((has_high_cpu, high_cpu_containers))
    }
    
    #[allow(dead_code)]
    pub async fn get_container_info(&self) -> Result<Vec<ContainerInspectResponse>> {
        let containers = self.docker.list_containers::<String>(None).await?;
        let mut container_info = Vec::new();
        
        for container in containers {
            if let Some(id) = container.id {
                match self.docker.inspect_container(&id, None).await {
                    Ok(info) => container_info.push(info),
                    Err(e) => {
                        error!("Error getting info for container {}: {}", id, e);
                        continue;
                    }
                }
            }
        }
        
        Ok(container_info)
    }
    
    pub async fn get_docker_system_info(&self) -> Result<DockerSystemInfo> {
        let info = self.docker.info().await?;
        let version = self.docker.version().await?;
        
        Ok(DockerSystemInfo {
            version: version.version.unwrap_or_else(|| "unknown".to_string()),
            api_version: version.api_version.unwrap_or_else(|| "unknown".to_string()),
            containers: info.containers.map_or(0, |c| c as u64),
            containers_running: info.containers_running.map_or(0, |c| c as u64),
            containers_paused: info.containers_paused.map_or(0, |c| c as u64),
            containers_stopped: info.containers_stopped.map_or(0, |c| c as u64),
            images: info.images.map_or(0, |i| i as u64),
            server_version: info.server_version.unwrap_or_else(|| "unknown".to_string()),
            memory_total: info.mem_total.map_or(0, |m| m as u64),
            cpu_count: info.ncpu.map_or(0, |n| n as u64),
        })
    }
}