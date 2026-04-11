# =========================
# Build stage
# =========================
FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies dulu
COPY Cargo.toml ./
RUN mkdir -p src/bin && \
    echo 'fn main() {}' > src/bin/main.rs && \
    cargo build --release && \
    rm -rf src

# Build aplikasi sebenarnya
COPY src ./src
RUN touch src/bin/main.rs && cargo build --release -p reengineering-tool-be

# =========================
# Runtime stage
# =========================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/reengineering-tool-be /app/reengineering-tool-be

EXPOSE 8080

ENTRYPOINT ["/app/reengineering-tool-be"]