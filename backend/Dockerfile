FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock* ./
COPY backend/shared ./shared
COPY backend/domain ./domain
COPY backend/infra ./infra
COPY backend/migration ./migration
COPY backend/migrations ./migrations
COPY backend/app ./app
COPY backend/api ./api
COPY backend/server ./server
COPY backend/cli ./cli

RUN cargo build --release --bin server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 wget \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 wiki \
    && useradd --uid 10001 --gid wiki --home-dir /nonexistent --shell /usr/sbin/nologin --no-create-home wiki \
    && mkdir -p /app /var/lib/wiki/uploads \
    && chown -R wiki:wiki /app /var/lib/wiki
WORKDIR /app
COPY --from=builder --chown=wiki:wiki /app/target/release/server /usr/local/bin/wiki-server
COPY --from=builder --chown=wiki:wiki /app/migrations ./migrations
ENV WIKI_MIGRATIONS_DIR=/app/migrations
USER wiki
EXPOSE 3456
CMD ["wiki-server"]
