#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}=================================${NC}"
    echo -e "${BLUE} $1 ${NC}"
    echo -e "${BLUE}=================================${NC}"
}

# Function to check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        print_error "This script should not be run as root. Run as regular user with sudo privileges."
        exit 1
    fi
}

# Function to update system
update_system() {
    print_header "Updating System"
    
    print_status "Updating package lists..."
    sudo apt update
    
    print_status "Upgrading packages..."
    sudo apt upgrade -y
    
    print_status "Installing basic utilities..."
    sudo apt install -y curl wget git htop vim unzip
    
    print_status "✅ System updated"
}

# Function to install Docker
install_docker() {
    print_header "Installing Docker"
    
    if command -v docker &> /dev/null; then
        print_status "Docker is already installed"
        docker --version
    else
        print_status "Installing Docker..."
        
        # Remove old versions
        sudo apt-get remove -y docker docker-engine docker.io containerd runc || true
        
        # Install dependencies
        sudo apt-get install -y \
            ca-certificates \
            curl \
            gnupg \
            lsb-release
        
        # Add Docker's official GPG key
        sudo mkdir -p /etc/apt/keyrings
        curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
        
        # Set up the repository
        echo \
            "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
            $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
        
        # Install Docker Engine
        sudo apt-get update
        sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
        
        print_status "✅ Docker installed"
    fi
}

# Function to install Docker Compose
install_docker_compose() {
    print_header "Installing Docker Compose"
    
    if command -v docker-compose &> /dev/null; then
        print_status "Docker Compose is already installed"
        docker-compose --version
    else
        print_status "Installing Docker Compose..."
        
        # Download Docker Compose
        sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        
        # Apply executable permissions
        sudo chmod +x /usr/local/bin/docker-compose
        
        print_status "✅ Docker Compose installed"
    fi
}

# Function to setup user permissions
setup_user_permissions() {
    print_header "Setting Up User Permissions"
    
    # Add user to docker group
    if ! groups $USER | grep -q docker; then
        print_status "Adding user to docker group..."
        sudo usermod -aG docker $USER
        print_warning "You need to log out and log back in for group changes to take effect"
    else
        print_status "User is already in docker group"
    fi
    
    # Create performance-monitor directory
    MONITOR_DIR="$HOME/performance-monitor"
    mkdir -p "$MONITOR_DIR"
    mkdir -p "$MONITOR_DIR/logs"
    mkdir -p "$MONITOR_DIR/backups"
    
    print_status "✅ User permissions setup completed"
}

# Function to setup firewall
setup_firewall() {
    print_header "Setting Up Firewall"
    
    # Check if UFW is installed
    if command -v ufw &> /dev/null; then
        print_status "Configuring UFW firewall..."
        
        # Allow SSH
        sudo ufw allow ssh
        
        # Allow Docker ports if needed
        sudo ufw allow 2376/tcp  # Docker daemon
        sudo ufw allow 7946/tcp  # Docker swarm
        sudo ufw allow 7946/udp  # Docker swarm
        sudo ufw allow 4789/udp  # Docker overlay network
        
        # Enable firewall
        sudo ufw --force enable
        
        print_status "✅ Firewall configured"
    else
        print_warning "UFW not found. Skipping firewall configuration"
    fi
}

# Function to setup log rotation
setup_log_rotation() {
    print_header "Setting Up Log Rotation"
    
    # Create logrotate configuration
    sudo tee /etc/logrotate.d/performance-monitor > /dev/null <<EOF
$HOME/performance-monitor/logs/*.log {
    daily
    missingok
    rotate 7
    compress
    delaycompress
    notifempty
    create 0644 $USER $USER
    postrotate
        # Send signal to performance-monitor to reopen log file
        docker exec performance-monitor pkill -USR1 performance-monitor 2>/dev/null || true
    endscript
}
EOF
    
    print_status "✅ Log rotation configured"
}

# Function to create systemd service for auto-start
create_systemd_service() {
    print_header "Creating Systemd Service"
    
    # Create systemd service for performance monitor
    sudo tee /etc/systemd/system/performance-monitor.service > /dev/null <<EOF
[Unit]
Description=Performance Monitor Service
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=$HOME/performance-monitor
ExecStart=/usr/local/bin/docker-compose up -d
ExecStop=/usr/local/bin/docker-compose down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF
    
    # Reload systemd and enable service
    sudo systemctl daemon-reload
    sudo systemctl enable performance-monitor.service
    
    print_status "✅ Systemd service created and enabled"
}

# Function to setup monitoring user
setup_monitoring_user() {
    print_header "Setting Up Monitoring User"
    
    # Create dedicated monitoring user (optional)
    if ! id "monitor" &>/dev/null; then
        print_status "Creating monitoring user..."
        sudo useradd -r -s /bin/false -m -d /var/lib/performance-monitor monitor
        sudo usermod -aG docker monitor
        print_status "✅ Monitoring user created"
    else
        print_status "Monitoring user already exists"
    fi
}

# Function to create backup script
create_backup_script() {
    print_header "Creating Backup Script"
    
    BACKUP_SCRIPT="$HOME/performance-monitor/backup.sh"
    
    cat > "$BACKUP_SCRIPT" << 'EOF'
#!/bin/bash

# Backup script for performance monitor
BACKUP_DIR="$HOME/performance-monitor/backups"
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_NAME="backup-$DATE"

mkdir -p "$BACKUP_DIR/$BACKUP_NAME"

# Backup configuration
cp -r "$HOME/performance-monitor/config.json" "$BACKUP_DIR/$BACKUP_NAME/" 2>/dev/null || true
cp -r "$HOME/performance-monitor/docker-compose.yml" "$BACKUP_DIR/$BACKUP_NAME/" 2>/dev/null || true

# Backup logs
cp -r "$HOME/performance-monitor/logs" "$BACKUP_DIR/$BACKUP_NAME/" 2>/dev/null || true

# Backup Docker images
if docker ps -a | grep -q "performance-monitor"; then
    cd "$HOME/performance-monitor"
    docker-compose save -o "$BACKUP_DIR/$BACKUP_NAME/performance-monitor-images.tar" 2>/dev/null || true
fi

# Compress backup
cd "$BACKUP_DIR"
tar -czf "$BACKUP_NAME.tar.gz" "$BACKUP_NAME"
rm -rf "$BACKUP_NAME"

echo "Backup created: $BACKUP_DIR/$BACKUP_NAME.tar.gz"

# Keep only last 5 backups
ls -t | tail -n +6 | xargs -r rm -f
EOF

    chmod +x "$BACKUP_SCRIPT"
    
    # Add to crontab for daily backups
    (crontab -l 2>/dev/null; echo "0 2 * * * $BACKUP_SCRIPT") | crontab -
    
    print_status "✅ Backup script created and scheduled"
}

# Function to show next steps
show_next_steps() {
    print_header "Setup Completed! 🎉"
    
    echo ""
    echo "Next steps:"
    echo "1. Log out and log back in for Docker group changes to take effect"
    echo "2. Clone the repository:"
    echo "   git clone https://github.com/your-username/performance-monitor.git"
    echo "   cd performance-monitor"
    echo ""
    echo "3. Copy and configure environment:"
    echo "   cp .env.example .env"
    echo "   nano .env  # Edit with your settings"
    echo ""
    echo "4. Deploy the application:"
    echo "   ./deploy.sh"
    echo ""
    echo "5. Check status:"
    echo "   ./deploy.sh status"
    echo ""
    echo "Useful commands:"
    echo "  View logs: ./deploy.sh logs"
    echo "  Check status: ./deploy.sh status"
    echo "  Stop services: ./deploy.sh stop"
    echo "  Restart services: ./deploy.sh restart"
    echo "  Create backup: ./deploy.sh backup"
    echo ""
    echo "Configuration files:"
    echo "  Config: $HOME/performance-monitor/config.json"
    echo "  Logs: $HOME/performance-monitor/logs/"
    echo "  Backups: $HOME/performance-monitor/backups/"
}

# Main setup function
main() {
    print_header "Performance Monitor Server Setup"
    echo "This script will set up your server for Performance Monitor deployment"
    echo ""
    
    check_root
    update_system
    install_docker
    install_docker_compose
    setup_user_permissions
    setup_firewall
    setup_log_rotation
    create_systemd_service
    setup_monitoring_user
    create_backup_script
    show_next_steps
}

# Handle script arguments
case "${1:-}" in
    "docker-only")
        install_docker
        install_docker_compose
        setup_user_permissions
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [COMMAND]"
        echo ""
        echo "Commands:"
        echo "  (no args)  Full server setup"
        echo "  docker-only Setup Docker only"
        echo "  help       Show this help"
        ;;
    "")
        main
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac