# Simple Rust build
FROM rust:latest

WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Create non-root user
RUN useradd --create-home --shell /bin/bash --user-group --uid 1001 argus

# Switch to non-root user
USER argus

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose port
EXPOSE 8080

# Run the service
CMD ["./target/release/argus"]