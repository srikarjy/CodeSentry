# Multi-stage build for development and production
FROM rust:1.75 as builder

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY tests/ tests/

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder stage
COPY --from=builder /app/target/release/rust-analysis-engine /usr/local/bin/rust-analysis-engine

# Expose the port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["rust-analysis-engine"]