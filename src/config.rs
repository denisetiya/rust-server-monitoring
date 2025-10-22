use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub monitoring: MonitoringConfig,
    pub email: EmailConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub cpu_threshold: f64,
    pub check_interval: u64,
    pub docker_stats_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub enabled: bool,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub sender_email: String,
    pub sender_password: String,
    pub recipient_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: String,
    pub max_size_mb: u32,
    pub backup_count: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            monitoring: MonitoringConfig {
                cpu_threshold: 80.0,
                check_interval: 300,
                docker_stats_timeout: 10,
            },
            email: EmailConfig {
                enabled: false,
                smtp_server: "smtp.gmail.com".to_string(),
                smtp_port: 587,
                sender_email: String::new(),
                sender_password: String::new(),
                recipient_email: String::new(),
            },
            logging: LoggingConfig {
                level: "INFO".to_string(),
                file: "monitoring.log".to_string(),
                max_size_mb: 10,
                backup_count: 5,
            },
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}