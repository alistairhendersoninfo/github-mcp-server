# Multi-stage build for minimal attack surface
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -m -u 1001 appuser

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY migrations/ ./migrations/

# Build the application with optimizations
RUN cargo build --release --locked

# Runtime stage with minimal base image
FROM debian:bookworm-slim

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN useradd -m -u 1001 -s /bin/bash appuser

# Create directories with proper permissions
RUN mkdir -p /app/data /app/config /app/web \
    && chown -R appuser:appuser /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/github-mcp-server /app/
COPY --from=builder /app/migrations/ /app/migrations/

# Copy web assets
COPY web/ /app/web/

# Copy configuration template
COPY config/ /app/config/

# Set proper permissions
RUN chmod +x /app/github-mcp-server \
    && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Set working directory
WORKDIR /app

# Expose port (non-privileged)
EXPOSE 8443

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8443/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:./data/github-mcp-server.db

# Run the application
CMD ["./github-mcp-server"]