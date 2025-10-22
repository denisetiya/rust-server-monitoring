#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="${REPO:-your-username/performance-monitor}"
REGISTRY="${REGISTRY:-ghcr.io}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
ENVIRONMENT="${ENVIRONMENT:-production}"
DEPLOY_DIR="${DEPLOY_DIR:-$HOME/performance-monitor}"
BACKUP_DIR="${BACKUP_DIR:-$HOME/performance-monitor-backups}"

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

# Function to check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    print_status "✅ Docker is installed"
    
    # Check if Docker Compose is installed
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    print_status "✅ Docker Compose is installed"
    
    # Check if user has Docker permissions
    if ! docker ps &> /dev/null; then
        print_error "User doesn't have Docker permissions. Please add user to docker group."
        exit 1
    fi
    print_status "✅ User has Docker permissions"
    
    # Check if curl is installed
    if ! command -v curl &> /dev/null; then
        print_error "curl is not installed. Please install curl first."
        exit 1
    fi
    print_status "✅ curl is installed"
}

# Function to create backup
create_backup() {
    print_header "Creating Backup"
    
    BACKUP_NAME="backup-$(date +%Y%m%d-%H%M%S)"
    BACKUP_PATH="$BACKUP_DIR/$BACKUP_NAME"
    
    mkdir -p "$BACKUP_PATH"
    
    if [ -d "$DEPLOY_DIR" ]; then
        print_status "Creating backup of current deployment..."
        
        # Backup configuration files
        cp -r "$DEPLOY_DIR/config.json" "$BACKUP_PATH/" 2>/dev/null || true
        cp -r "$DEPLOY_DIR/docker-compose.yml" "$BACKUP_PATH/" 2>/dev/null || true
        cp -r "$DEPLOY_DIR/logs" "$BACKUP_PATH/" 2>/dev/null || true
        
        # Backup Docker images
        if docker ps -a | grep -q "performance-monitor"; then
            docker-compose -f "$DEPLOY_DIR/docker-compose.yml" save -o "$BACKUP_PATH/performance-monitor-images.tar" 2>/dev/null || true
        fi
        
        print_status "✅ Backup created: $BACKUP_PATH"
    else
        print_warning "No existing deployment to backup"
    fi
}

# Function to setup directories
setup_directories() {
    print_header "Setting Up Directories"
    
    mkdir -p "$DEPLOY_DIR"
    mkdir -p "$DEPLOY_DIR/logs"
    mkdir -p "$BACKUP_DIR"
    
    print_status "✅ Directories created"
}

# Function to download latest files
download_files() {
    print_header "Downloading Latest Files"
    
    cd "$DEPLOY_DIR"
    
    # Download docker-compose.yml
    print_status "Downloading docker-compose.yml..."
    curl -o docker-compose.yml "https://raw.githubusercontent.com/$REPO/main/docker-compose.yml"
    
    # Download config template if not exists
    if [ ! -f "config.json" ]; then
        print_status "Downloading config.json template..."
        curl -o config.json "https://raw.githubusercontent.com/$REPO/main/config.json"
        print_warning "Please edit config.json with your settings before starting the service"
    fi
    
    print_status "✅ Files downloaded"
}

# Function to update configuration
update_configuration() {
    print_header "Updating Configuration"
    
    cd "$DEPLOY_DIR"
    
    # Update image tag in docker-compose.yml
    sed -i "s|image: performance-monitor:latest|image: $REGISTRY/$REPO:$IMAGE_TAG|g" docker-compose.yml
    
    # Update container names for different environments
    if [ "$ENVIRONMENT" != "production" ]; then
        sed -i "s|container_name: performance-monitor|container_name: performance-monitor-$ENVIRONMENT|g" docker-compose.yml
        sed -i "s|container_name: monitor-log-rotate|container_name: monitor-log-rotate-$ENVIRONMENT|g" docker-compose.yml
        sed -i "s|./logs|./logs-$ENVIRONMENT|g" docker-compose.yml
        mkdir -p "logs-$ENVIRONMENT"
    fi
    
    print_status "✅ Configuration updated"
}

# Function to pull and deploy
deploy_services() {
    print_header "Deploying Services"
    
    cd "$DEPLOY_DIR"
    
    # Pull latest images
    print_status "Pulling latest images..."
    docker-compose pull
    
    # Stop existing services
    print_status "Stopping existing services..."
    docker-compose down 2>/dev/null || true
    
    # Start services
    print_status "Starting services..."
    docker-compose up -d
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 15
    
    print_status "✅ Services deployed"
}

# Function to verify deployment
verify_deployment() {
    print_header "Verifying Deployment"
    
    cd "$DEPLOY_DIR"
    
    # Check service status
    print_status "Checking service status..."
    docker-compose ps
    
    # Check if main service is running
    if docker-compose ps | grep -q "Up"; then
        print_status "✅ Services are running"
    else
        print_error "❌ Services are not running properly"
        docker-compose logs --tail=20
        exit 1
    fi
    
    # Check logs
    print_status "Recent logs:"
    docker-compose logs --tail=10 performance-monitor
    
    print_status "✅ Deployment verified successfully"
}

# Function to cleanup old backups
cleanup_backups() {
    print_header "Cleaning Up Old Backups"
    
    # Keep only last 5 backups
    cd "$BACKUP_DIR"
    ls -t | tail -n +6 | xargs -r rm -rf
    
    print_status "✅ Old backups cleaned up"
}

# Function to show deployment info
show_deployment_info() {
    print_header "Deployment Information"
    
    echo "Environment: $ENVIRONMENT"
    echo "Repository: $REPO"
    echo "Image Tag: $IMAGE_TAG"
    echo "Deploy Directory: $DEPLOY_DIR"
    echo "Backup Directory: $BACKUP_DIR"
    echo ""
    echo "Useful commands:"
    echo "  View logs: docker-compose -f $DEPLOY_DIR/docker-compose.yml logs -f"
    echo "  Check status: docker-compose -f $DEPLOY_DIR/docker-compose.yml ps"
    echo "  Stop services: docker-compose -f $DEPLOY_DIR/docker-compose.yml down"
    echo "  Restart services: docker-compose -f $DEPLOY_DIR/docker-compose.yml restart"
    echo ""
    echo "Configuration file: $DEPLOY_DIR/config.json"
    echo "Log files: $DEPLOY_DIR/logs/"
}

# Main deployment function
main() {
    print_header "Performance Monitor Deployment Script"
    echo "Environment: $ENVIRONMENT"
    echo "Repository: $REPO"
    echo "Image Tag: $IMAGE_TAG"
    echo ""
    
    check_prerequisites
    create_backup
    setup_directories
    download_files
    update_configuration
    deploy_services
    verify_deployment
    cleanup_backups
    show_deployment_info
    
    print_header "Deployment Completed Successfully! 🎉"
}

# Handle script arguments
case "${1:-}" in
    "backup")
        create_backup
        ;;
    "status")
        if [ -f "$DEPLOY_DIR/docker-compose.yml" ]; then
            cd "$DEPLOY_DIR"
            docker-compose ps
        else
            print_error "Deployment not found in $DEPLOY_DIR"
        fi
        ;;
    "logs")
        if [ -f "$DEPLOY_DIR/docker-compose.yml" ]; then
            cd "$DEPLOY_DIR"
            docker-compose logs -f
        else
            print_error "Deployment not found in $DEPLOY_DIR"
        fi
        ;;
    "stop")
        if [ -f "$DEPLOY_DIR/docker-compose.yml" ]; then
            cd "$DEPLOY_DIR"
            docker-compose down
            print_status "Services stopped"
        else
            print_error "Deployment not found in $DEPLOY_DIR"
        fi
        ;;
    "restart")
        if [ -f "$DEPLOY_DIR/docker-compose.yml" ]; then
            cd "$DEPLOY_DIR"
            docker-compose restart
            print_status "Services restarted"
        else
            print_error "Deployment not found in $DEPLOY_DIR"
        fi
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [COMMAND]"
        echo ""
        echo "Commands:"
        echo "  (no args)  Full deployment"
        echo "  backup     Create backup only"
        echo "  status     Show service status"
        echo "  logs       Show service logs"
        echo "  stop       Stop services"
        echo "  restart    Restart services"
        echo "  help       Show this help"
        echo ""
        echo "Environment variables:"
        echo "  REPO           Repository name (default: your-username/performance-monitor)"
        echo "  REGISTRY       Container registry (default: ghcr.io)"
        echo "  IMAGE_TAG      Image tag (default: latest)"
        echo "  ENVIRONMENT    Environment (default: production)"
        echo "  DEPLOY_DIR     Deployment directory (default: ~/performance-monitor)"
        echo "  BACKUP_DIR     Backup directory (default: ~/performance-monitor-backups)"
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