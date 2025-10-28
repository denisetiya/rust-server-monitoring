# Docker & Server Performance Monitoring

Sistem monitoring untuk Docker container dan server yang akan mengirimkan email alert ketika penggunaan CPU melebihi 80%. Dibuat dengan **Rust** untuk performa tinggi dan resource usage yang optimal, dilengkapi dengan **CI/CD pipeline** untuk automasi deployment.

## 🚀 Fitur

- ✅ Monitoring penggunaan CPU server real-time
- ✅ Monitoring penggunaan CPU per Docker container
- ✅ Email alert ketika CPU usage > 80%
- ✅ **Daily email limit** - maksimal 1 email per hari untuk mencegah spam
- ✅ **Detailed logging** dengan timestamp dan container details
- ✅ **Fixed CPU calculation bug** - CPU usage tidak lagi selalu 0%
- ✅ Menampilkan container dengan penggunaan CPU tertinggi
- ✅ Konfigurasi yang mudah disesuaikan
- ✅ Docker container deployment
- ✅ Log rotation dan management
- ✅ High performance dengan Rust
- ✅ **CI/CD Pipeline** dengan GitHub Actions
- ✅ **Automated deployment** ke production/staging
- ✅ **Multi-environment support**

## 📁 Struktur Project

```
performance-monitoring/
├── README.md                 # Dokumentasi
├── config.json              # File konfigurasi
├── config.production.json   # Config untuk production
├── .env.example             # Environment variables template
├── build.sh                 # Build script
├── deploy.sh                # Deployment script
├── server-setup.sh          # Server setup script
├── docker-compose.yml       # Docker Compose configuration
├── Dockerfile               # Docker image build
├── logrotate.conf           # Log rotation configuration
├── .dockerignore           # Docker ignore file
├── Cargo.toml              # Rust dependencies
├── Cargo.lock              # Rust dependency lock file
├── .github/
│   └── workflows/
│       └── ci-cd.yml       # GitHub Actions CI/CD pipeline
└── src/
    ├── main.rs              # Main application
    ├── config.rs            # Configuration module
    ├── server_monitor.rs    # Server monitoring module
    ├── docker_monitor.rs    # Docker monitoring module
    └── email_notifier.rs    # Email notification module
```

## 🔄 CI/CD Pipeline

### GitHub Actions Workflow

Pipeline otomatis yang berjalan pada setiap push/PR:

1. **Test & Build**
   - Rust formatting check (`cargo fmt`)
   - Clippy linting (`cargo clippy`)
   - Unit tests (`cargo test`)
   - Release build (`cargo build --release`)

2. **Security Scan**
   - Trivy vulnerability scanner
   - SARIF report upload

3. **Docker Build & Push**
   - Multi-platform build (linux/amd64, linux/arm64)
   - Push ke GitHub Container Registry (GHCR)
   - Semantic versioning tags

4. **Automated Deployment**
   - **Production**: Push ke `main` branch
   - **Staging**: Push ke `develop` branch
   - **Release**: Tagged releases

### Environment Setup

#### GitHub Secrets

**📋 Lihat daftar lengkap secrets yang dibutuhkan:** [GITHUB_SECRETS.md](GITHUB_SECRETS.md)

Set up secrets di GitHub repository settings:

```bash
# Production Server
PROD_HOST=your-server.com
PROD_USER=deploy
PROD_PORT=22
PROD_SSH_KEY=-----BEGIN OPENSSH PRIVATE KEY-----
...

# Staging Server
STAGING_HOST=staging-server.com
STAGING_USER=deploy
STAGING_PORT=22
STAGING_SSH_KEY=-----BEGIN OPENSSH PRIVATE KEY-----
...
```

📖 **Dokumentasi lengkap setup secrets:** [GITHUB_SECRETS.md](GITHUB_SECRETS.md)

## 🐳 Docker Deployment

### Quick Start dengan CI/CD

1. **Fork repository**
2. **Set up GitHub secrets**
3. **Push ke main branch** → Otomatis deploy ke production

```bash
git clone https://github.com/denisetiya/rust-server-monitoring.git
cd performance-monitor
git remote add upstream https://github.com/original-owner/rust-server-monitoring.git

# Set up environment
cp .env.example .env
nano .env  # Edit dengan credentials Anda

# Push untuk trigger deployment
git add .
git commit -m "Initial deployment setup"
git push origin main
```

### Manual Deployment

```bash
# Build image
./build.sh

# Deploy ke server
./deploy.sh

# Atau gunakan Docker Compose
docker-compose up -d
```

## 🖥️ Server Setup

### Automated Server Setup

```bash
# Jalankan di server target
curl -fsSL https://raw.githubusercontent.com/your-username/performance-monitor/main/server-setup.sh | bash

# Atau download dan jalankan manual
wget https://raw.githubusercontent.com/your-username/rust-server-monitoring/main/server-setup.sh
chmod +x server-setup.sh
./server-setup.sh
```

### Manual Server Setup

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add user to docker group
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Clone repository
git clone https://github.com/denisetiya/rust-server-monitoring.git
cd performance-monitor

# Deploy
./deploy.sh
```

## 🦀 Local Development

### Installation

1. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Build dan jalankan:**
   ```bash
   # Build
   cargo build --release
   
   # Jalankan
   ./target/release/performance-monitor
   ```

### Available Commands

```bash
# Show system status
performance-monitor --status

# Test email configuration
performance-monitor --test-email

# Run single monitoring check
performance-monitor

# Run continuous monitoring
performance-monitor --continuous

# Custom configuration file
performance-monitor --config /path/to/config.json

# Custom monitoring interval (seconds)
performance-monitor --continuous --interval 60

# Show help
performance-monitor --help
```

## ⚙️ Konfigurasi

### Environment Variables (.env)

```bash
# Email Configuration
SENDER_EMAIL=your-email@gmail.com
SENDER_PASSWORD=your-app-password
RECIPIENT_EMAIL=alert-email@example.com

# Server Configuration
PROD_HOST=your-server.com
PROD_USER=deploy
PROD_PORT=22
PROD_SSH_KEY_PATH=~/.ssh/id_rsa

# Repository Configuration
REPO=your-username/performance-monitor
REGISTRY=ghcr.io

# Deployment Configuration
ENVIRONMENT=production
IMAGE_TAG=latest
DEPLOY_DIR=~/performance-monitor
BACKUP_DIR=~/performance-monitor-backups
```

### 🔐 Configuration Security

**⚠️ IMPORTANT:** `config.json` contains sensitive information and should NEVER be committed to version control!

#### Setup Configuration

1. **Copy example configuration:**
   ```bash
   cp config.json.example config.json
   ```

2. **Edit with your actual credentials:**
   ```bash
   nano config.json
   ```

3. **Configuration structure:**
   ```json
   {
     "monitoring": {
       "cpu_threshold": 80,
       "check_interval": 300,
       "docker_stats_timeout": 10
     },
     "email": {
       "enabled": true,
       "smtp_server": "smtp.gmail.com",
       "smtp_port": 587,
       "sender_email": "your-email@gmail.com",
       "sender_password": "your-app-password",
       "recipient_email": "alert-email@example.com",
       "daily_limit": true
     },
     "logging": {
       "level": "INFO",
       "file": "monitoring.log",
       "max_size_mb": 10,
       "backup_count": 5
     }
   }
   ```
   
   **New Email Configuration Options:**
   - `daily_limit`: (boolean) Batasi email alert maksimal 1x per hari untuk mencegah spam
   - Set `daily_limit: false` jika ingin email dikirim setiap kali ada alert

#### Multi-Environment Configuration

- **Development:** `config.json` (local use only, ignored by git)
- **Production:** `config.production.json` (generated from GitHub secrets)
- **Staging:** `config.staging.json` (based on config.json.example)

#### CI/CD Configuration Management

The CI/CD pipeline automatically creates production configuration from GitHub secrets:

```yaml
# GitHub Secrets Setup
EMAIL_ENABLED=true
SMTP_SERVER=smtp.gmail.com
SMTP_PORT=587
SENDER_EMAIL=your-email@gmail.com
SENDER_PASSWORD=your-app-password
RECIPIENT_EMAIL=alert@example.com
LOG_LEVEL=INFO
```

## 🔄 Deployment Commands

### Deployment Script Usage

```bash
# Full deployment
./deploy.sh

# Check deployment status
./deploy.sh status

# View logs
./deploy.sh logs

# Stop services
./deploy.sh stop

# Restart services
./deploy.sh restart

# Create backup
./deploy.sh backup

# Show help
./deploy.sh help
```

### Environment-specific Deployment

```bash
# Production
ENVIRONMENT=production ./deploy.sh

# Staging
ENVIRONMENT=staging ./deploy.sh

# Custom image tag
IMAGE_TAG=v1.2.3 ./deploy.sh
```

## 📊 Monitoring Output

### System Status Example

```
============================================================
SYSTEM STATUS - 2024-01-15 10:30:45
============================================================

🖥️  SERVER:
   CPU Usage: 45.2%
   Memory Usage: 62.8%
   Disk Usage: 78.5%

🐳 DOCKER:
   Running Containers: 4
   Total Containers: 4

   Top CPU Containers:
   1. app-web: 25.3% CPU
   2. web-profile: 12.1% CPU
   3. mysql_1: 5.2% CPU
   4. phpmyadmin_1: 2.6% CPU

============================================================
```

### Email Alert Example

Ketika CPU usage > 80%, email alert akan dikirim dengan:

- 📊 Server CPU usage percentage
- 🐳 Daftar container dengan CPU usage tinggi
- 🕐 Timestamp alert
- 📈 Resource usage details

## 🔧 Troubleshooting

### Common Issues

1. **Docker Socket Permission:**
   ```bash
   sudo usermod -aG docker $USER
   ```

2. **Email Not Sending:**
   - Check SMTP configuration
   - Verify app password (not regular password)
   - Check firewall settings

3. **Build Issues:**
   ```bash
   # Clean build
   cargo clean
   cargo build --release
   ```

4. **Deployment Issues:**
   ```bash
   # Check deployment status
   ./deploy.sh status
   
   # View logs
   ./deploy.sh logs
   
   # Check SSH connection
   ssh -i ~/.ssh/id_rsa user@server
   ```

### Debug Mode

```bash
# Rust version dengan debug log
RUST_LOG=debug performance-monitor --status

# Docker debug
docker-compose logs -f performance-monitor
```

## 🔒 Security Considerations

- Use read-only Docker socket mount (`:ro`)
- Run with non-root user in container
- Secure email credentials dengan environment variables
- Regular log rotation untuk mencegah disk penuh
- SSH key authentication untuk deployment
- GitHub secrets untuk sensitive data

## 🚀 Performance

### Rust Advantages

- **Memory Safety**: Tidak ada memory leaks
- **High Performance**: CPU dan memory usage rendah
- **Concurrency**: Async/await untuk non-blocking operations
- **Zero-cost Abstractions**: Efficient runtime performance

### Resource Usage

- CPU: < 1% during normal operation
- Memory: ~10-20MB baseline
- Network: Minimal (SMTP connections only)

## 🔄 CI/CD Best Practices

### Branch Strategy

- `main`: Production-ready code
- `develop`: Staging environment
- `feature/*`: Feature development
- `hotfix/*`: Emergency fixes

### Deployment Flow

1. **Development**: `feature/*` → `develop` (staging)
2. **Release**: `develop` → `main` (production)
3. **Hotfix**: `main` → `hotfix/*` → `main`

### Monitoring & Alerts

- CI/CD pipeline status notifications
- Deployment success/failure alerts
- Rollback capabilities
- Health checks

## 🆕 Recent Improvements (v0.2.0)

### Bug Fixes
- **Fixed CPU Usage Bug**: CPU usage tidak lagi selalu menampilkan 0%
- **Proper CPU Calculation**: Implementasi perhitungan CPU yang akurat menggunakan Docker Stats API

### New Features
- **Daily Email Limit**: Email alert maksimal 1x per hari untuk mencegah spam
- **Enhanced Logging**: Log detail dengan timestamp dan informasi container
- **Top CPU Containers**: Log menampilkan 10 container dengan CPU usage tertinggi
- **High CPU Detection**: Log khusus untuk containers dengan CPU > 50%

### Configuration Updates
- Added `daily_limit` field to email configuration
- Backward compatible with existing configurations

### How Daily Limit Works
1. Saat high CPU detected, sistem selalu log detail ke file
2. Cek apakah sudah kirim email hari ini via `last_alert_date.txt`
3. Jika belum, kirim email dan update tanggal
4. Jika sudah, skip email tapi tetap log
5. Reset keesokan hari

### Logging Improvements
```
=== CPU USAGE ALERT DETAILS ===
Alert Time: 2025-10-23 12:28:10 UTC
Server CPU Usage: 20.90%
Total Running Containers: 5

=== TOP CPU CONSUMING CONTAINERS ===
1. nginx-proxy [a1b2c3d4e5f6]: 15.2% CPU, 128.5 MB RAM (12.1%)
2. redis-cache [f6e5d4c3b2a1]: 8.7% CPU, 64.2 MB RAM (6.4%)
...
```

For detailed information, see [CPU_MONITORING_IMPROVEMENTS.md](./CPU_MONITORING_IMPROVEMENTS.md)

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

### Development Workflow

```bash
# Setup development environment
git clone https://github.com/denisetiya/rust-server-monitoring.git
cd rust-server-monitoring
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Run locally
cargo run -- --status
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Support

Jika mengalami masalah:

1. **Check logs:** `tail -f logs/monitoring.log`
2. **Test configuration:** `performance-monitor --test-email`
3. **Verify Docker access:** `docker ps`
4. **Check system status:** `performance-monitor --status`
5. **Debug mode:** `RUST_LOG=debug performance-monitor --status`
6. **Check deployment:** `./deploy.sh status`

### CI/CD Issues

1. **GitHub Actions**: Check Actions tab in repository
2. **Deployment failures**: Check deployment logs
3. **SSH issues**: Verify SSH keys and permissions
4. **Docker issues**: Check Docker daemon status

---

**Performance Monitor** - 🚀 Built with Rust & CI/CD for reliable and high-performance system monitoring