use lettre::{
    Message, SmtpTransport, Transport,
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
};
use chrono::Utc;
use crate::config::{Config, EmailConfig};
use crate::docker_monitor::ContainerStats;
use log::{info, error, warn};

pub struct EmailNotifier {
    config: EmailConfig,
    enabled: bool,
}

impl EmailNotifier {
    pub fn new(config: Config) -> Self {
        let email_config = config.email.clone();
        let enabled = email_config.enabled;
        
        if enabled {
            if email_config.sender_email.is_empty() 
                || email_config.sender_password.is_empty() 
                || email_config.recipient_email.is_empty() {
                warn!("Email configuration incomplete. Email notifications disabled.");
                Self {
                    config: email_config,
                    enabled: false,
                }
            } else {
                info!("Email notifier initialized");
                Self {
                    config: email_config,
                    enabled: true,
                }
            }
        } else {
            info!("Email notifications disabled");
            Self {
                config: email_config,
                enabled: false,
            }
        }
    }
    
    pub async fn send_alert(&self, subject: &str, message: &str) -> bool {
        if !self.enabled {
            info!("Email notifications disabled. Skipping alert.");
            return false;
        }
        
        let email = Message::builder()
            .from(self.config.sender_email.parse().unwrap())
            .to(self.config.recipient_email.parse().unwrap())
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(self.strip_html_tags(message))
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(message.to_string())
                    )
            );
        
        if let Ok(email) = email {
            let creds = Credentials::new(
                self.config.sender_email.clone(),
                self.config.sender_password.clone()
            );
            
            let mailer = SmtpTransport::relay(&self.config.smtp_server)
                .unwrap()
                .port(self.config.smtp_port)
                .credentials(creds)
                .build();
            
            match mailer.send(&email) {
                Ok(_) => {
                    info!("Alert email sent successfully to {}", self.config.recipient_email);
                    true
                }
                Err(e) => {
                    error!("Failed to send email alert: {}", e);
                    false
                }
            }
        } else {
            error!("Failed to build email message");
            false
        }
    }
    
    pub async fn send_cpu_alert(&self, server_cpu: f64, high_cpu_containers: &[ContainerStats]) -> bool {
        let subject = format!("🚨 HIGH CPU USAGE ALERT - {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
        
        let message = format!(
            r#"
            <html>
            <body>
                <h2>🚨 HIGH CPU USAGE ALERT</h2>
                <p><strong>Time:</strong> {}</p>
                
                <h3>📊 Server CPU Usage</h3>
                <p><strong>Current CPU Usage:</strong> <span style="color: red; font-size: 18px; font-weight: bold;">{:.2}%</span></p>
                <p><strong>Threshold:</strong> 80%</p>
                
                <h3>🐳 High CPU Docker Containers</h3>
                {}
                
                <br>
                <p><em>This is an automated alert from your Docker & Server Performance Monitoring System.</em></p>
                <p><em>Please check your server and containers immediately.</em></p>
            </body>
            </html>
            "#,
            Utc::now().format("%Y-%m-%d %H:%M:%S"),
            server_cpu,
            self.format_container_table(high_cpu_containers)
        );
        
        self.send_alert(&subject, &message).await
    }
    
    pub async fn send_container_cpu_alert(&self, high_cpu_containers: &[ContainerStats]) -> bool {
        let subject = format!("🐳 HIGH CONTAINER CPU ALERT - {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
        
        let message = format!(
            r#"
            <html>
            <body>
                <h2>🐳 HIGH CONTAINER CPU USAGE ALERT</h2>
                <p><strong>Time:</strong> {}</p>
                
                <h3>🔥 High CPU Docker Containers</h3>
                {}
                <br>
                <p><em>This is an automated alert from your Docker & Server Performance Monitoring System.</em></p>
                <p><em>Please check the highlighted containers immediately.</em></p>
            </body>
            </html>
            "#,
            Utc::now().format("%Y-%m-%d %H:%M:%S"),
            self.format_detailed_container_table(high_cpu_containers)
        );
        
        self.send_alert(&subject, &message).await
    }
    
    pub async fn send_test_email(&self) -> bool {
        let subject = "🧪 Test Email - Docker & Server Performance Monitoring".to_string();
        
        let message = format!(
            r#"
            <html>
            <body>
                <h2>🧪 Test Email</h2>
                <p>This is a test email from your Docker & Server Performance Monitoring System.</p>
                <p><strong>Time:</strong> {}</p>
                <p>If you receive this email, your email configuration is working correctly.</p>
                <br>
                <p><em>System is ready to send alerts when CPU usage exceeds the threshold.</em></p>
            </body>
            </html>
            "#,
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        
        self.send_alert(&subject, &message).await
    }
    
    fn format_container_table(&self, containers: &[ContainerStats]) -> String {
        if containers.is_empty() {
            return "<p>No specific containers with high CPU usage detected.</p>".to_string();
        }
        
        let mut table = String::from(
            "<table border='1' style='border-collapse: collapse; width: 100%;'>"
        );
        table.push_str("<tr style='background-color: #f2f2f2;'>");
        table.push_str("<th style='padding: 8px; text-align: left;'>Container Name</th>");
        table.push_str("<th style='padding: 8px; text-align: left;'>CPU Usage</th>");
        table.push_str("<th style='padding: 8px; text-align: left;'>Memory Usage</th>");
        table.push_str("<th style='padding: 8px; text-align: left;'>Image</th>");
        table.push_str("</tr>");
        
        for container in containers {
            table.push_str("<tr>");
            table.push_str(&format!("<td style='padding: 8px;'>{}</td>", container.name));
            table.push_str(&format!(
                "<td style='padding: 8px; color: red; font-weight: bold;'>{:.2}%</td>", 
                container.cpu_usage
            ));
            table.push_str(&format!("<td style='padding: 8px;'>{:.2}%</td>", container.memory_percent));
            table.push_str(&format!("<td style='padding: 8px;'>{}</td>", container.image));
            table.push_str("</tr>");
        }
        
        table.push_str("</table>");
        table
    }
    
    fn format_detailed_container_table(&self, containers: &[ContainerStats]) -> String {
        let mut table = String::from(
            "<table border='1' style='border-collapse: collapse; width: 100%;'>"
        );
        table.push_str("<tr style='background-color: #f2f2f2;'>");
        table.push_str("<th style='padding: 8px; text-align: left;'>Container Name</th>");
        table.push_str("<th style='padding: 8px; text-align: left;'>CPU Usage</th>");
        table.push_str("<th style='padding: 8px; text-align: left;'>Memory Usage</th>");
        table.push_str("<th style='padding: 8px; text-align: left;'>Image</th>");
        table.push_str("<th style='padding: 8px; text-align: left;'>Status</th>");
        table.push_str("</tr>");
        
        for container in containers {
            table.push_str("<tr>");
            table.push_str(&format!("<td style='padding: 8px;'>{}</td>", container.name));
            table.push_str(&format!(
                "<td style='padding: 8px; color: red; font-weight: bold;'>{:.2}%</td>", 
                container.cpu_usage
            ));
            table.push_str(&format!("<td style='padding: 8px;'>{:.2}%</td>", container.memory_percent));
            table.push_str(&format!("<td style='padding: 8px;'>{}</td>", container.image));
            table.push_str(&format!("<td style='padding: 8px;'>{}</td>", container.status));
            table.push_str("</tr>");
        }
        
        table.push_str("</table>");
        table
    }
    
    fn strip_html_tags(&self, html: &str) -> String {
        // Simple HTML tag stripper
        let mut result = String::new();
        let mut in_tag = false;
        
        for ch in html.chars() {
            if ch == '<' {
                in_tag = true;
            } else if ch == '>' {
                in_tag = false;
            } else if !in_tag {
                result.push(ch);
            }
        }
        
        // Clean up extra whitespace
        result.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}