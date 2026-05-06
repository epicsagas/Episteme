# Syntagma Dockerfile
# Multi-stage Rust build for production deployment

# ---------------------------------------------------------------------------
# Build stage
# ---------------------------------------------------------------------------
FROM rust:1.85-slim AS builder

WORKDIR /usr/src/syntagma

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release

# ---------------------------------------------------------------------------
# Runtime stage
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/syntagma/target/release/syntagma /usr/local/bin/syntagma
COPY --from=builder /usr/src/syntagma/target/release/syntagma-mcp /usr/local/bin/syntagma-mcp

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

CMD ["syntagma", "api"]
