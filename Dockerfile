# Multi-stage build for Rust performance monitoring application

# Stage 1: Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false monitor

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/performance-monitor /usr/local/bin/performance-monitor

# Copy configuration file
COPY config.json /app/config.json

# Create log directory
RUN mkdir -p /app/logs && chown -R monitor:monitor /app

# Switch to non-root user
USER monitor

# Expose health check endpoint (if needed in future)
# EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/performance-monitor --status || exit 1

# Default command
CMD ["/usr/local/bin/performance-monitor", "--continuous"]