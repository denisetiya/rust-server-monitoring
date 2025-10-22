# GitHub Secrets Configuration

Berikut adalah daftar lengkap GitHub Secrets yang perlu diatur di repository Anda untuk CI/CD pipeline berjalan dengan baik.

## 🔧 Cara Setup GitHub Secrets

1. Buka repository GitHub Anda
2. Go to **Settings** → **Secrets and variables** → **Actions**
3. Klik **New repository secret**
4. Masukkan nama dan value sesuai daftar di bawah

## 📋 Daftar GitHub Secrets

### 🖥️ Production Server Secrets

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `PROD_HOST` | IP address atau domain production server | `192.168.1.100` atau `your-server.com` |
| `PROD_USER` | Username untuk SSH ke production server | `deploy` atau `ubuntu` |
| `PROD_PORT` | Port SSH (default: 22) | `22` |
| `PROD_SSH_KEY` | Private SSH key untuk production server | `-----BEGIN OPENSSH PRIVATE KEY-----\n...` |

### 🧪 Staging Server Secrets

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `STAGING_HOST` | IP address atau domain staging server | `192.168.1.200` atau `staging.your-server.com` |
| `STAGING_USER` | Username untuk SSH ke staging server | `deploy` atau `ubuntu` |
| `STAGING_PORT` | Port SSH (default: 22) | `22` |
| `STAGING_SSH_KEY` | Private SSH key untuk staging server | `-----BEGIN OPENSSH PRIVATE KEY-----\n...` |

### 📧 Email Configuration Secrets

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `SENDER_EMAIL` | Email pengirim alerts | `your-email@gmail.com` |
| `SENDER_PASSWORD` | App password untuk email (bukan password biasa) | `abcd efgh ijkl mnop` |
| `RECIPIENT_EMAIL` | Email penerima alerts | `alerts@your-company.com` |

### 🐳 Docker Registry Secrets (Opsional)

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `DOCKER_USERNAME` | Username untuk Docker registry (jika private) | `your-docker-username` |
| `DOCKER_PASSWORD` | Password atau token untuk Docker registry | `your-docker-token` |

### 🔔 Notification Secrets (Opsional)

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `SLACK_WEBHOOK` | Slack webhook URL untuk notifikasi | `https://hooks.slack.com/services/...` |
| `DISCORD_WEBHOOK` | Discord webhook URL untuk notifikasi | `https://discord.com/api/webhooks/...` |

## 🔑 SSH Key Setup

### 1. Generate SSH Key Pair

```bash
# Generate SSH key untuk deployment
ssh-keygen -t ed25519 -C "performance-monitor-deploy" -f ~/.ssh/performance-monitor

# Ini akan membuat:
# Private key: ~/.ssh/performance-monitor
# Public key: ~/.ssh/performance-monitor.pub
```

### 2. Add Public Key ke Server

```bash
# Copy public key ke server
ssh-copy-id -i ~/.ssh/performance-monitor.pub user@server

# Atau manual:
cat ~/.ssh/performance-monitor.pub | ssh user@server "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"
```

### 3. Add Private Key ke GitHub Secrets

```bash
# Tampilkan private key (copy seluruh output)
cat ~/.ssh/performance-monitor
```

Output akan seperti ini:
```
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAlwAAAADzc2gtZW
QyNTUxOQAAACBpVQ5QJH8wE8K9vN5qL9mX9vN5qL9mX9vN5qL9mX9vN5qL9mX9vN5qL9mX9vN5qL9mX9
...
-----END OPENSSH PRIVATE KEY-----
```

Copy seluruh output dan paste ke GitHub Secrets dengan nama `PROD_SSH_KEY` atau `STAGING_SSH_KEY`.

## 📧 Email Setup (Gmail)

### 1. Enable 2-Factor Authentication

1. Go to [Google Account settings](https://myaccount.google.com/)
2. Security → 2-Step Verification
3. Enable 2FA

### 2. Create App Password

1. Go to [Google App Passwords](https://myaccount.google.com/apppasswords)
2. Select app: **Mail**
3. Select device: **Other (Custom name)**
4. Name: **Performance Monitor**
5. Generate password

Password yang dihasilkan akan seperti: `abcd efgh ijkl mnop`

### 3. Add Email Secrets ke GitHub

| Secret Name | Value |
|-------------|-------|
| `SENDER_EMAIL` | `your-email@gmail.com` |
| `SENDER_PASSWORD` | `abcd efgh ijkl mnop` |
| `RECIPIENT_EMAIL` | `alerts@your-company.com` |

## 🔍 Testing Configuration

### 1. Test SSH Connection

```bash
# Test SSH ke production server
ssh -i ~/.ssh/performance-monitor deploy@PROD_HOST

# Test SSH ke staging server
ssh -i ~/.ssh/performance-monitor deploy@STAGING_HOST
```

### 2. Test Email Configuration

```bash
# Clone repository
git clone https://github.com/your-username/performance-monitor.git
cd performance-monitor

# Setup environment
cp .env.example .env
nano .env  # Edit dengan email settings

# Test email
cargo build --release
./target/release/performance-monitor --test-email
```

### 3. Test CI/CD Pipeline

1. Push ke `develop` branch untuk test staging deployment
2. Push ke `main` branch untuk test production deployment

## 🛡️ Security Best Practices

### SSH Key Security

- ✅ Gunakan SSH key yang berbeda untuk production dan staging
- ✅ Set permissions: `chmod 600 ~/.ssh/performance-monitor`
- ✅ Limit SSH key usage hanya untuk deployment user
- ✅ Gunakan `ed25519` algorithm (lebih aman dari RSA)

### Email Security

- ✅ Gunakan App Password (bukan password Gmail biasa)
- ✅ Enable 2-Factor Authentication
- ✅ Gunakan email account khusus untuk alerts
- ✅ Regular rotation password

### GitHub Secrets Security

- ✅ Jangan commit secrets ke repository
- ✅ Gunakan descriptive names untuk secrets
- ✅ Regular audit secrets yang tidak digunakan
- ✅ Limit access ke repository settings

## 🚨 Troubleshooting

### SSH Connection Issues

```bash
# Debug SSH connection
ssh -v -i ~/.ssh/performance-monitor deploy@server

# Check SSH key permissions
ls -la ~/.ssh/performance-monitor

# Fix permissions jika needed
chmod 600 ~/.ssh/performance-monitor
chmod 644 ~/.ssh/performance-monitor.pub
```

### Email Issues

- ✅ Pastikan 2FA enabled
- ✅ Gunakan App Password (bukan password biasa)
- ✅ Check SMTP settings di configuration
- ✅ Test dengan manual email client

### CI/CD Issues

- ✅ Check GitHub Actions logs
- ✅ Verify secret names (case-sensitive)
- ✅ Test SSH connection manual
- ✅ Check Docker daemon running di server

## 📝 Checklist Sebelum Deployment

- [ ] SSH keys generated dan added ke server
- [ ] SSH private keys added ke GitHub Secrets
- [ ] Email 2FA enabled dan App Password created
- [ ] Email secrets added ke GitHub Secrets
- [ ] Server Docker daemon running
- [ ] Firewall allows SSH (port 22) dan Docker ports
- [ ] Test SSH connection manual
- [ ] Test email configuration manual
- [ ] Repository pushed ke GitHub
- [ ] GitHub Actions workflow enabled

## 🔄 Maintenance

### Regular Tasks

1. **Monthly**: Rotate SSH keys jika needed
2. **Quarterly**: Update App passwords
3. **As needed**: Update server credentials
4. **Annually**: Audit semua secrets

### Monitoring

- Monitor GitHub Actions logs untuk failures
- Check email alerts working properly
- Verify SSH connections stable
- Update secrets jika ada security alerts

---

**Catatan**: Simpan file ini di repository sebagai referensi, tapi jangan include actual secrets atau credentials di file ini!