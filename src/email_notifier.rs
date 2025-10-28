use crate::config::{Config, EmailConfig};
use crate::docker_monitor::ContainerStats;
use crate::timezone_utils;
use chrono::NaiveDate;
use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use log::{error, info, warn};
use std::fs;

pub struct EmailNotifier {
    config: EmailConfig,
    enabled: bool,
    last_alert_file: String,
}

impl EmailNotifier {
    pub fn new(config: Config) -> Self {
        let email_config = config.email.clone();
        let enabled = email_config.enabled;
        let last_alert_file = "last_alert_date.txt".to_string();

        if enabled {
            if email_config.sender_email.is_empty()
                || email_config.sender_password.is_empty()
                || email_config.recipient_email.is_empty()
            {
                warn!("Email configuration incomplete. Email notifications disabled.");
                Self {
                    config: email_config,
                    enabled: false,
                    last_alert_file,
                }
            } else {
                info!("Email notifications enabled");
                Self {
                    config: email_config,
                    enabled: true,
                    last_alert_file,
                }
            }
        } else {
            info!("Email notifications disabled in configuration");
            Self {
                config: email_config,
                enabled: false,
                last_alert_file,
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn should_send_alert(&self) -> bool {
        if !self.config.daily_limit {
            return true; // Always send if daily limit is disabled
        }

        let today = timezone_utils::today_wib();

        if let Ok(content) = fs::read_to_string(&self.last_alert_file) {
            if let Ok(last_date) = content.trim().parse::<NaiveDate>() {
                if last_date >= today {
                    info!(
                        "Email alert already sent today ({}), skipping to avoid spam",
                        today
                    );
                    return false;
                }
            }
        }

        true
    }

    fn update_last_alert_date(&self) {
        let today = timezone_utils::today_wib();
        if let Err(e) = fs::write(&self.last_alert_file, today.to_string()) {
            error!("Failed to update last alert date: {}", e);
        } else {
            info!("Updated last alert date to: {}", today);
        }
    }

    fn log_cpu_usage_details(&self, server_cpu: f64, all_containers: &[ContainerStats]) {
        let timestamp = timezone_utils::now_wib();

        info!("=== CPU USAGE ALERT DETAILS ===");
        info!("Alert Time: {}", timestamp.format("%Y-%m-%d %H:%M:%S WIB"));
        info!("Server CPU Usage: {:.2}%", server_cpu);
        info!("Total Running Containers: {}", all_containers.len());

        if !all_containers.is_empty() {
            info!("=== TOP CPU CONSUMING CONTAINERS ===");
            for (i, container) in all_containers.iter().take(10).enumerate() {
                info!(
                    "{}. {} [{}]: {:.2}% CPU, {:.1} MB RAM ({:.1}%)",
                    i + 1,
                    container.name,
                    &container.id[..std::cmp::min(12, container.id.len())],
                    container.cpu_usage,
                    container.memory_usage as f64 / 1024.0 / 1024.0,
                    container.memory_percent
                );
            }
        }

        let high_cpu_containers: Vec<_> = all_containers
            .iter()
            .filter(|c| c.cpu_usage > 50.0)
            .collect();

        if !high_cpu_containers.is_empty() {
            warn!("=== CONTAINERS ABOVE 50% CPU ===");
            for container in high_cpu_containers {
                warn!(
                    "⚠️  {} [{}]: {:.2}% CPU - NEEDS ATTENTION",
                    container.name,
                    &container.id[..std::cmp::min(12, container.id.len())],
                    container.cpu_usage
                );
            }
        }

        info!("=== END CPU USAGE ALERT DETAILS ===");
    }

    pub async fn send_alert(&self, subject: &str, message: &str) -> bool {
        if !self.enabled {
            info!("Email notifications disabled. Skipping alert.");
            return false;
        }

        info!(
            "Attempting to send email alert to {} via {}:{}",
            self.config.recipient_email, self.config.smtp_server, self.config.smtp_port
        );

        if let Ok(email) = Message::builder()
            .from(self.config.sender_email.parse().unwrap())
            .to(self.config.recipient_email.parse().unwrap())
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(self.strip_html_tags(message)),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(message.to_string()),
                    ),
            )
        {
            let creds = Credentials::new(
                self.config.sender_email.clone(),
                self.config.sender_password.clone(),
            );

            // Configure SMTP transport with STARTTLS for Gmail compatibility
            let mailer = SmtpTransport::starttls_relay(&self.config.smtp_server)
                .unwrap()
                .port(self.config.smtp_port)
                .credentials(creds)
                .build();

            match mailer.send(&email) {
                Ok(_) => {
                    info!(
                        "Alert email sent successfully to {}",
                        self.config.recipient_email
                    );
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

    pub async fn send_cpu_alert(
        &self,
        server_cpu: f64,
        high_cpu_containers: &[ContainerStats],
        all_containers: &[ContainerStats],
    ) -> bool {
        // Always log the details regardless of email sending
        self.log_cpu_usage_details(server_cpu, all_containers);

        // If email is not enabled, consider logging successful
        if !self.enabled {
            return true; // Logging was successful, email not expected
        }

        // Check if we should send email today
        if !self.should_send_alert() {
            return true; // Don't send email today, but not an error
        }

        let subject = format!(
            "🚨 HIGH CPU USAGE ALERT - {}",
            timezone_utils::format_wib_alert()
        );

        // Sort all containers by CPU usage (descending)
        let mut sorted_containers = all_containers.to_vec();
        sorted_containers.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());

        // Get top 10 containers
        let top_containers = &sorted_containers[..std::cmp::min(10, sorted_containers.len())];

        let message = format!(
            r#"
            <html>
            <head>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 20px; }}
                    table {{ border-collapse: collapse; width: 100%; margin: 10px 0; }}
                    th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
                    th {{ background-color: #f2f2f2; }}
                    .high-cpu {{ background-color: #ffe6e6; }}
                    .warning {{ background-color: #fff3cd; }}
                    .normal {{ background-color: #e8f5e8; }}
                    .alert-header {{ color: #d32f2f; font-weight: bold; }}
                    .summary-box {{ background-color: #f8f9fa; padding: 15px; border-left: 4px solid #dc3545; margin: 10px 0; }}
                </style>
            </head>
            <body>
                <h2 class="alert-header">🚨 HIGH CPU USAGE ALERT</h2>
                <p><strong>Alert Time:</strong> {}</p>
                
                <div class="summary-box">
                    <h3>📊 CRITICAL - Server CPU Usage</h3>
                    <p><strong>Current CPU Usage:</strong> <span style="color: red; font-size: 20px; font-weight: bold;">{:.2}%</span></p>
                    <p><strong>Alert Threshold:</strong> 80.00%</p>
                    <p><strong>High CPU Containers:</strong> {}</p>
                    <p><strong>Total Running Containers:</strong> {}</p>
                </div>
                
                <h3>🔥 Top 10 CPU Consuming Containers</h3>
                {}
                
                {}
                
                <h3>💡 Recommended Actions</h3>
                <ul>
                    <li><strong>Immediate:</strong> Check top CPU containers for issues</li>
                    <li><strong>Investigate:</strong> Look for memory leaks or infinite loops</li>
                    <li><strong>Scale:</strong> Consider horizontal scaling if traffic is high</li>
                    <li><strong>Resources:</strong> Monitor memory usage and disk I/O</li>
                    <li><strong>Logs:</strong> Check container logs for errors: <code>docker logs [container-name]</code></li>
                </ul>
                
                <hr>
                <p><em>🤖 This is an automated alert from your Docker & Server Performance Monitoring System.</em></p>
                <p><em>⚡ System monitoring frequency: Every 5 minutes</em></p>
                <p><em>📧 Daily email limit: Only one alert email per day to prevent spam</em></p>
                <p><em>📋 Detailed logs are always recorded regardless of email sending</em></p>
                <p><em>🔧 To modify thresholds or disable alerts, update your monitoring configuration.</em></p>
            </body>
            </html>
            "#,
            timezone_utils::format_wib_log(),
            server_cpu,
            high_cpu_containers.len(),
            all_containers.len(),
            self.format_detailed_container_table(top_containers),
            if !high_cpu_containers.is_empty() {
                format!(
                    "<h3>⚠️ Containers Above Threshold ({})</h3>\n{}",
                    high_cpu_containers.len(),
                    self.format_high_cpu_containers(high_cpu_containers)
                )
            } else {
                "<p><em>✅ No containers are above the CPU threshold.</em></p>".to_string()
            }
        );

        let email_sent = self.send_alert(&subject, &message).await;

        if email_sent {
            // Update the last alert date only if email was successfully sent
            self.update_last_alert_date();
            info!("CPU alert email sent successfully and daily limit updated");
        }

        email_sent
    }

    pub async fn send_container_cpu_alert(&self, high_cpu_containers: &[ContainerStats]) -> bool {
        // If email is not enabled, return true (no error expected)
        if !self.enabled {
            info!("Container CPU alert would be triggered, but email notifications are disabled");
            return true;
        }

        let subject = format!(
            "🐳 HIGH CONTAINER CPU ALERT - {}",
            timezone_utils::format_wib_alert()
        );

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
            timezone_utils::format_wib_alert(),
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
            timezone_utils::format_wib_alert()
        );

        self.send_alert(&subject, &message).await
    }

    fn format_detailed_container_table(&self, containers: &[ContainerStats]) -> String {
        if containers.is_empty() {
            return "<p><em>No containers running.</em></p>".to_string();
        }

        let mut table = String::from(
            r#"
            <table>
                <thead>
                    <tr>
                        <th>#</th>
                        <th>Container Name</th>
                        <th>Image</th>
                        <th>Status</th>
                        <th>CPU %</th>
                        <th>Memory Usage</th>
                        <th>Memory %</th>
                        <th>Ports</th>
                    </tr>
                </thead>
                <tbody>
            "#,
        );

        for (index, container) in containers.iter().enumerate() {
            let row_class = if container.cpu_usage > 80.0 {
                "high-cpu"
            } else if container.cpu_usage > 50.0 {
                "warning"
            } else {
                "normal"
            };

            let memory_mb = container.memory_usage as f64 / 1024.0 / 1024.0;
            let memory_limit_mb = container.memory_limit as f64 / 1024.0 / 1024.0;
            let ports_str = if container.ports.is_empty() {
                "-".to_string()
            } else {
                container.ports.join(", ")
            };

            table.push_str(&format!(
                r#"
                <tr class="{}">
                    <td>{}</td>
                    <td><strong>{}</strong></td>
                    <td>{}</td>
                    <td>{}</td>
                    <td><strong>{:.1}%</strong></td>
                    <td>{:.1} MB / {:.1} MB</td>
                    <td>{:.1}%</td>
                    <td><small>{}</small></td>
                </tr>
                "#,
                row_class,
                index + 1,
                self.truncate_name(&container.name),
                self.truncate_image(&container.image),
                container.status,
                container.cpu_usage,
                memory_mb,
                memory_limit_mb,
                container.memory_percent,
                ports_str
            ));
        }

        table.push_str("</tbody></table>");
        table
    }

    fn format_high_cpu_containers(&self, containers: &[ContainerStats]) -> String {
        if containers.is_empty() {
            return "<p><em>✅ All containers are operating within normal CPU limits.</em></p>"
                .to_string();
        }

        let mut content = String::from(
            "<div style='background-color: #ffe6e6; padding: 15px; border-radius: 5px;'>",
        );

        for container in containers {
            let memory_mb = container.memory_usage as f64 / 1024.0 / 1024.0;
            content.push_str(&format!(
                r#"
                <div style="margin-bottom: 10px; border-left: 4px solid #d32f2f; padding-left: 10px;">
                    <h4 style="margin: 0; color: #d32f2f;">🚨 {}</h4>
                    <p style="margin: 5px 0;">
                        <strong>CPU:</strong> <span style="color: red; font-weight: bold;">{:.1}%</span> | 
                        <strong>Memory:</strong> {:.1} MB ({:.1}%) | 
                        <strong>Image:</strong> {} | 
                        <strong>Status:</strong> {}
                    </p>
                    <p style="margin: 5px 0; font-size: 12px; color: #666;">
                        <strong>Container ID:</strong> {} | <strong>Ports:</strong> {}
                    </p>
                </div>
                "#,
                self.truncate_name(&container.name),
                container.cpu_usage,
                memory_mb,
                container.memory_percent,
                self.truncate_image(&container.image),
                container.status,
                &container.id[..std::cmp::min(12, container.id.len())],
                if container.ports.is_empty() { "None".to_string() } else { container.ports.join(", ") }
            ));
        }

        content.push_str("</div>");
        content
    }

    fn truncate_name(&self, name: &str) -> String {
        if name.len() > 25 {
            format!("{}...", &name[..22])
        } else {
            name.to_string()
        }
    }

    fn truncate_image(&self, image: &str) -> String {
        // Remove registry prefix and truncate
        let parts: Vec<&str> = image.split('/').collect();
        let image_name = parts.last().unwrap_or(&image);

        if image_name.len() > 30 {
            format!("{}...", &image_name[..27])
        } else {
            image_name.to_string()
        }
    }

    fn strip_html_tags(&self, html: &str) -> String {
        // Simple HTML tag stripper for plain text email
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
        result
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
