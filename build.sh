#!/bin/bash

set -e

echo "🚀 Building Docker & Server Performance Monitor (Rust Version)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    print_error "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Check if Rust is installed (for local development)
if command -v cargo &> /dev/null; then
    print_status "Rust detected. You can also build locally with 'cargo build --release'"
else
    print_warning "Rust not detected. Docker build will be used."
fi

# Create necessary directories
print_status "Creating necessary directories..."
mkdir -p logs
mkdir -p config

# Check if config.json exists
if [ ! -f "config.json" ]; then
    print_warning "config.json not found. Creating default configuration..."
    cat > config.json << EOF
{
  "monitoring": {
    "cpu_threshold": 80,
    "check_interval": 300,
    "docker_stats_timeout": 10
  },
  "email": {
    "enabled": false,
    "smtp_server": "smtp.gmail.com",
    "smtp_port": 587,
    "sender_email": "your-email@gmail.com",
    "sender_password": "your-app-password",
    "recipient_email": "alert-email@example.com"
  },
  "logging": {
    "level": "INFO",
    "file": "monitoring.log",
    "max_size_mb": 10,
    "backup_count": 5
  }
}
EOF
    print_warning "Please edit config.json with your email settings before running the monitor."
fi

# Build Docker image
print_status "Building Docker image..."
docker build -t performance-monitor:latest .

if [ $? -eq 0 ]; then
    print_status "Docker image built successfully!"
else
    print_error "Failed to build Docker image."
    exit 1
fi

# Test the build
print_status "Testing the build..."
docker run --rm \
    -v /var/run/docker.sock:/var/run/docker.sock:ro \
    -v $(pwd)/config.json:/app/config.json:ro \
    performance-monitor:latest \
    --status

if [ $? -eq 0 ]; then
    print_status "Build test successful!"
else
    print_warning "Build test failed. This might be due to missing Docker socket access."
fi

print_status "Build completed successfully!"
print_status ""
print_status "Next steps:"
print_status "1. Edit config.json with your email settings"
print_status "2. Run with Docker Compose: docker-compose up -d"
print_status "3. Or run directly: docker run -d --name performance-monitor \\"
print_status "   -v /var/run/docker.sock:/var/run/docker.sock:ro \\"
print_status "   -v \$(pwd)/config.json:/app/config.json:ro \\"
print_status "   -v \$(pwd)/logs:/app/logs \\"
print_status "   performance-monitor:latest"
print_status ""
print_status "To test email configuration:"
print_status "docker run --rm -v \$(pwd)/config.json:/app/config.json:ro performance-monitor:latest --test-email"