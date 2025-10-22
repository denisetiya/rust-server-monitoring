use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use chrono::{DateTime, Utc};
use crate::config::Config;
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f64,
    pub memory_usage: MemoryStats,
    pub disk_usage: DiskStats,
    pub load_average: LoadAverage,
    pub system_info: SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStats {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAverage {
    pub one_min: f64,
    pub five_min: f64,
    pub fifteen_min: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub cpu_count: usize,
    pub cpu_brand: String,
    pub total_memory: u64,
    pub boot_time: DateTime<Utc>,
}

pub struct ServerMonitor {
    system: System,
    config: Config,
}

impl ServerMonitor {
    pub fn new(config: Config) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system,
            config,
        }
    }
    
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    pub fn get_cpu_usage(&mut self) -> f64 {
        self.refresh();
        self.system.global_cpu_info().cpu_usage().into()
    }
    
    pub fn get_memory_usage(&mut self) -> MemoryStats {
        self.refresh();
        let memory = self.system.total_memory();
        let used = self.system.used_memory();
        let available = self.system.available_memory();
        
        MemoryStats {
            total: memory,
            used,
            available,
            percent: (used as f64 / memory as f64) * 100.0,
        }
    }
    
    pub fn get_disk_usage(&mut self) -> DiskStats {
        self.refresh();
        // Get root disk usage
        if let Some(disk) = self.system.disks().first() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            
            DiskStats {
                total,
                used,
                available,
                percent: (used as f64 / total as f64) * 100.0,
            }
        } else {
            DiskStats {
                total: 0,
                used: 0,
                available: 0,
                percent: 0.0,
            }
        }
    }
    
    pub fn get_load_average(&self) -> LoadAverage {
        // For Linux systems, we can read from /proc/loadavg
        if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = loadavg.split_whitespace().collect();
            if parts.len() >= 3 {
                return LoadAverage {
                    one_min: parts[0].parse().unwrap_or(0.0),
                    five_min: parts[1].parse().unwrap_or(0.0),
                    fifteen_min: parts[2].parse().unwrap_or(0.0),
                };
            }
        }
        
        // Fallback values
        LoadAverage {
            one_min: 0.0,
            five_min: 0.0,
            fifteen_min: 0.0,
        }
    }
    
    pub fn get_system_info(&self) -> SystemInfo {
        let boot_time = DateTime::from_timestamp(self.system.boot_time() as i64, 0)
            .unwrap_or_else(|| Utc::now());
        
        SystemInfo {
            hostname: self.system.name().unwrap_or_else(|| "Unknown".to_string()),
            os: self.system.long_os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel: self.system.kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            cpu_count: self.system.cpus().len(),
            cpu_brand: self.system.cpus()
                .first()
                .map(|cpu| cpu.brand().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            total_memory: self.system.total_memory(),
            boot_time,
        }
    }
    
    pub fn check_cpu_threshold(&mut self) -> (bool, f64) {
        let cpu_usage = self.get_cpu_usage();
        let threshold = self.config.monitoring.cpu_threshold;
        
        if cpu_usage > threshold {
            warn!("High CPU usage detected: {:.2}% (threshold: {:.2}%)", cpu_usage, threshold);
            (true, cpu_usage)
        } else {
            info!("CPU usage is normal: {:.2}%", cpu_usage);
            (false, cpu_usage)
        }
    }
    
    pub fn get_full_stats(&mut self) -> ServerStats {
        ServerStats {
            timestamp: Utc::now(),
            cpu_usage: self.get_cpu_usage(),
            memory_usage: self.get_memory_usage(),
            disk_usage: self.get_disk_usage(),
            load_average: self.get_load_average(),
            system_info: self.get_system_info(),
        }
    }
}