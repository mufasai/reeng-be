FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/reengineering-tool-be /app/reengineering-tool-be

# Tambah ini untuk debug
RUN ldd /app/reengineering-tool-be || true
RUN /app/reengineering-tool-be --version || true

EXPOSE 8080

ENTRYPOINT ["/app/reengineering-tool-be"]