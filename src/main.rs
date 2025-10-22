use clap::{Arg, Command};
use log::{info, error, warn};
use env_logger::Env;
use std::time::Duration;
use anyhow::Result;

mod config;
mod server_monitor;
mod docker_monitor;
mod email_notifier;

use config::Config;
use server_monitor::ServerMonitor;
use docker_monitor::DockerMonitor;
use email_notifier::EmailNotifier;

struct PerformanceMonitor {
    config: Config,
    server_monitor: ServerMonitor,
    docker_monitor: DockerMonitor,
    email_notifier: EmailNotifier,
}

impl PerformanceMonitor {
    async fn new(config_path: &str) -> Result<Self> {
        // Load configuration
        let config = match Config::load_from_file(config_path) {
            Ok(config) => {
                info!("Configuration loaded from {}", config_path);
                config
            }
            Err(e) => {
                warn!("Failed to load config from {}: {}. Using default configuration.", config_path, e);
                Config::default()
            }
        };
        
        // Initialize monitors
        let server_monitor = ServerMonitor::new(config.clone());
        let docker_monitor = match DockerMonitor::new(config.clone()).await {
            Ok(monitor) => {
                info!("Docker monitor initialized successfully");
                monitor
            }
            Err(e) => {
                error!("Failed to initialize Docker monitor: {}", e);
                return Err(e);
            }
        };
        let email_notifier = EmailNotifier::new(config.clone());
        
        info!("Performance Monitor initialized");
        info!("CPU Threshold: {}%", config.monitoring.cpu_threshold);
        
        Ok(Self {
            config,
            server_monitor,
            docker_monitor,
            email_notifier,
        })
    }
    
    async fn check_server_cpu(&mut self) -> (bool, f64) {
        info!("Checking server CPU usage...");
        
        let (is_high, cpu_usage) = self.server_monitor.check_cpu_threshold();
        
        if is_high {
            warn!("High CPU usage detected: {:.2}%", cpu_usage);
            
            // Get high CPU containers
            let (_, high_cpu_containers) = self.docker_monitor
                .check_container_cpu_threshold(50.0)
                .await
                .unwrap_or((false, vec![]));
            
            // Send alert
            let alert_sent = self.email_notifier.send_cpu_alert(cpu_usage, &high_cpu_containers).await;
            if alert_sent {
                info!("CPU alert email sent successfully");
            } else {
                error!("Failed to send CPU alert email");
            }
        } else {
            info!("Server CPU usage is normal: {:.2}%", cpu_usage);
        }
        
        (is_high, cpu_usage)
    }
    
    async fn check_container_cpu(&self) -> (bool, Vec<docker_monitor::ContainerStats>) {
        info!("Checking Docker container CPU usage...");
        
        match self.docker_monitor.check_container_cpu_threshold(self.config.monitoring.cpu_threshold).await {
            Ok((is_high, high_cpu_containers)) => {
                if is_high {
                    warn!("High CPU usage detected in {} containers", high_cpu_containers.len());
                    
                    // Send alert
                    let alert_sent = self.email_notifier.send_container_cpu_alert(&high_cpu_containers).await;
                    if alert_sent {
                        info!("Container CPU alert email sent successfully");
                    } else {
                        error!("Failed to send container CPU alert email");
                    }
                } else {
                    info!("All containers have normal CPU usage");
                }
                
                (is_high, high_cpu_containers)
            }
            Err(e) => {
                error!("Error checking container CPU: {}", e);
                (false, vec![])
            }
        }
    }
    
    async fn run_monitoring(&mut self) -> Result<bool> {
        info!("Starting monitoring check...");
        
        // Check server CPU
        let (server_high, server_cpu) = self.check_server_cpu().await;
        
        // Check container CPU
        let (container_high, high_containers) = self.check_container_cpu().await;
        
        // Log summary
        info!("Monitoring check completed. Server CPU: {:.2}%, High CPU containers: {}", 
              server_cpu, high_containers.len());
        
        Ok(server_high || container_high)
    }
    
    async fn print_status_summary(&mut self) -> Result<()> {
        let server_stats = self.server_monitor.get_full_stats();
        let docker_stats = self.docker_monitor.get_container_stats().await.unwrap_or_default();
        let docker_info = self.docker_monitor.get_docker_system_info().await.unwrap_or_default();
        
        println!("\n{}", "=".repeat(60));
        println!("SYSTEM STATUS - {}", server_stats.timestamp.format("%Y-%m-%d %H:%M:%S"));
        println!("{}", "=".repeat(60));
        
        // Server status
        println!("\n🖥️  SERVER:");
        println!("   CPU Usage: {:.2}%", server_stats.cpu_usage);
        println!("   Memory Usage: {:.2}%", server_stats.memory_usage.percent);
        println!("   Disk Usage: {:.2}%", server_stats.disk_usage.percent);
        
        // Docker status
        println!("\n🐳 DOCKER:");
        println!("   Running Containers: {}", docker_stats.len());
        println!("   Total Containers: {}", docker_info.containers);
        
        if !docker_stats.is_empty() {
            println!("\n   Top CPU Containers:");
            for (i, container) in docker_stats.iter().take(5).enumerate() {
                println!("   {}. {}: {:.2}% CPU", i + 1, container.name, container.cpu_usage);
            }
        }
        
        println!("\n{}", "=".repeat(60));
        
        Ok(())
    }
    
    async fn run_continuous(&mut self) -> Result<()> {
        let interval = Duration::from_secs(self.config.monitoring.check_interval);
        
        info!("Starting continuous monitoring with {:?} interval...", interval);
        
        loop {
            match self.run_monitoring().await {
                Ok(alert_triggered) => {
                    if alert_triggered {
                        println!("⚠️  High CPU usage detected! Check your email for alerts.");
                    } else {
                        println!("✅ All systems normal.");
                    }
                }
                Err(e) => {
                    error!("Error during monitoring check: {}", e);
                }
            }
            
            tokio::time::sleep(interval).await;
        }
    }
    
    async fn test_email(&self) -> Result<()> {
        info!("Testing email configuration...");
        
        let success = self.email_notifier.send_test_email().await;
        if success {
            println!("✅ Test email sent successfully!");
        } else {
            println!("❌ Failed to send test email. Check your configuration.");
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("performance-monitor")
        .version("0.1.0")
        .author("Performance Monitor")
        .about("Docker & Server Performance Monitor written in Rust")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .default_value("config.json")
        )
        .arg(
            Arg::new("status")
                .short('s')
                .long("status")
                .help("Show current system status")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("test-email")
                .short('t')
                .long("test-email")
                .help("Test email configuration")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("continuous")
                .short('r')
                .long("continuous")
                .help("Run continuous monitoring")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();
    
    // Initialize logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    let config_path = matches.get_one::<String>("config").unwrap();
    
    // Initialize monitor
    let mut monitor = PerformanceMonitor::new(config_path).await?;
    
    if matches.get_flag("test-email") {
        monitor.test_email().await?;
    } else if matches.get_flag("status") {
        monitor.print_status_summary().await?;
    } else if matches.get_flag("continuous") {
        monitor.run_continuous().await?;
    } else {
        // Run single monitoring check
        match monitor.run_monitoring().await {
            Ok(alert_triggered) => {
                if alert_triggered {
                    println!("⚠️  High CPU usage detected! Check your email for alerts.");
                } else {
                    println!("✅ All systems normal.");
                }
            }
            Err(e) => {
                error!("Error during monitoring: {}", e);
            }
        }
    }
    
    Ok(())
}