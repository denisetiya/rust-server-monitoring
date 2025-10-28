# CPU Monitoring Improvements

## Masalah yang Diperbaiki

### 1. Bug CPU Usage Selalu 0
**Masalah**: Fungsi `calculate_cpu_usage` dalam `docker_monitor.rs` selalu mengembalikan 0.0
**Solusi**: Implementasi perhitungan CPU yang proper menggunakan Docker Stats API

**Perubahan di `src/docker_monitor.rs`:**
```rust
fn calculate_cpu_usage(&self, stats: &bollard::container::Stats) -> Result<f64> {
    // Calculate CPU usage based on Docker stats
    let cpu_delta = stats.cpu_stats.cpu_usage.total_usage.saturating_sub(
        stats.precpu_stats.cpu_usage.total_usage
    ) as f64;
    
    let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0).saturating_sub(
        stats.precpu_stats.system_cpu_usage.unwrap_or(0)
    ) as f64;
    
    let online_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
    
    if system_delta > 0.0 && cpu_delta >= 0.0 {
        let cpu_percent = (cpu_delta / system_delta) * online_cpus * 100.0;
        Ok(cpu_percent.min(100.0).max(0.0))
    } else {
        // Fallback: simple calculation based on available data
        let total_usage = stats.cpu_stats.cpu_usage.total_usage as f64;
        if total_usage > 0.0 {
            let estimated_percent = (total_usage / 1_000_000_000.0).min(100.0).max(0.0);
            Ok(estimated_percent)
        } else {
            Ok(0.0)
        }
    }
}
```

### 2. Email Dikirim Terus-Menerus 
**Masalah**: Email alert dikirim setiap kali ada CPU tinggi tanpa batasan
**Solusi**: Implementasi sistem daily limit untuk email notifications

**Perubahan di `src/config.rs`:**
- Menambahkan field `daily_limit: bool` ke `EmailConfig`

**Perubahan di `src/email_notifier.rs`:**
- Menambahkan field `last_alert_file: String` untuk tracking tanggal terakhir
- Fungsi `should_send_alert()` untuk mengecek apakah boleh kirim email
- Fungsi `update_last_alert_date()` untuk update tanggal terakhir kirim email

### 3. Logging Detail CPU Usage
**Masalah**: Tidak ada log detail jam berapa dan container mana yang menggunakan CPU tertinggi
**Solusi**: Implementasi logging komprehensif dengan detail timestamp dan top containers

**Perubahan di `src/email_notifier.rs`:**
```rust
fn log_cpu_usage_details(&self, server_cpu: f64, all_containers: &[ContainerStats]) {
    let timestamp = Utc::now();
    
    info!("=== CPU USAGE ALERT DETAILS ===");
    info!("Alert Time: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
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
```

## Fitur Baru

### 1. Daily Email Limit
- Email alert hanya dikirim maksimal 1 kali per hari
- File `last_alert_date.txt` menyimpan tanggal terakhir email dikirim
- Logging tetap berjalan normal meskipun email tidak dikirim
- Dapat di-disable dengan setting `daily_limit: false` di config

### 2. Enhanced Logging
- Log detail timestamp saat terjadi high CPU usage
- Log top 10 containers dengan CPU usage tertinggi
- Log khusus untuk containers dengan CPU > 50%
- Log selalu berjalan bahkan jika email tidak dikirim

### 3. Improved Email Content
- Menampilkan informasi bahwa sistem menggunakan daily email limit
- Menjelaskan bahwa detailed logs selalu dicatat
- Lebih informatif dengan timestamp dan container details

## Konfigurasi

Update file `config.json`:
```json
{
  "email": {
    "enabled": true,
    "smtp_server": "smtp.gmail.com",
    "smtp_port": 587,
    "sender_email": "your-email@gmail.com",
    "sender_password": "your-app-password", 
    "recipient_email": "alert-email@example.com",
    "daily_limit": true
  }
}
```

## Testing

1. **Test Status**: `./target/debug/performance-monitor --status`
2. **Test Single Check**: `./target/debug/performance-monitor`  
3. **Test Continuous**: `./target/debug/performance-monitor --continuous`
4. **Test Email**: `./target/debug/performance-monitor --test-email`

## Log Files

- `monitoring.log`: Log utama aplikasi
- `last_alert_date.txt`: File tracking tanggal terakhir email alert dikirim

## Cara Kerja Daily Limit

1. Saat terjadi high CPU usage, sistem selalu melakukan logging detail
2. Sistem cek apakah sudah kirim email hari ini dengan membaca `last_alert_date.txt`
3. Jika belum pernah kirim hari ini, email dikirim dan tanggal di-update
4. Jika sudah pernah kirim hari ini, email di-skip tapi logging tetap jalan
5. Keesokan harinya, sistem akan kirim email lagi jika ada alert

## Manfaat

1. **Tidak Spam Email**: Maksimal 1 email per hari
2. **Logging Lengkap**: Selalu ada catatan detail di log file
3. **CPU Monitoring Akurat**: Bug CPU 0% sudah diperbaiki
4. **Easy Troubleshooting**: Log menunjukkan container mana yang bermasalah