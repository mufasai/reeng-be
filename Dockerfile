# =========================
# Stage 1: Cache dependencies
# =========================
FROM rust:1.90-alpine3.20 AS deps

RUN apk add --no-cache build-base musl-dev pkgconf && \
    rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY Cargo.toml ./
RUN mkdir -p src/bin && echo 'fn main() { println!("prebuild binary"); }' > src/bin/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm -rf src

# =========================
# Stage 2: Build app
# =========================
FROM deps AS builder

COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl -p reengineering-tool-be

# =========================
# Stage 3: Runtime
# =========================
FROM alpine:3.20

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/reengineering-tool-be /app/reengineering-tool-be

EXPOSE 8080

ENTRYPOINT ["/app/reengineering-tool-be"]