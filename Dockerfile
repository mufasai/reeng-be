# =========================
# Build stage
# =========================
FROM rust:1-slim-bookworm AS builder

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

EXPOSE 3001

# Railway will set PORT env variable, app will use it
ENV PORT=3001
ENV RUST_LOG=info
RUN echo '#!/bin/sh\necho "==== CONTAINER STARTING ===="\nenv\nls -la /app\necho "==== STARTING RUST APP ===="\nexec /app/reengineering-tool-be' > /app/start.sh && \
    chmod +x /app/start.sh /app/reengineering-tool-be

ENTRYPOINT ["/app/start.sh"]