# =========================
# Build stage
# =========================
FROM rust:latest AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies dulu
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && \
    echo 'fn main() {}' > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build aplikasi sebenarnya
COPY src ./src
RUN cargo build --release -p reengineering-tool-be

# =========================
# Runtime stage
# =========================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    file \
    binutils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/reengineering-tool-be /app/reengineering-tool-be

# Debug: cek binary compatibility
RUN file /app/reengineering-tool-be && ldd /app/reengineering-tool-be || true

EXPOSE 8080

ENTRYPOINT ["/app/reengineering-tool-be"]