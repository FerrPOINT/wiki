# syntax=docker/dockerfile:1
# Workspace-root context: product backend and the verified sibling SDK.
FROM rust:1.88.0-slim-bookworm AS manifests
ARG PDLC_RUN_SLOT_SHARING=locked
ARG TARGETARCH
ARG CARGO_REGISTRY_CACHE_ID=pdlc-operator-downloads-v1-cargo188
ARG CARGO_BUILD_JOBS=2

RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} apt-get update && apt-get install -y pkg-config libssl-dev unzip curl unzip && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# infra/Cargo.toml: sdlc-auth-core = { path = "../../../services-base/..." }
# /app/infra/../../../services-base == /services-base
# sdlc-auth-core inherits workspace.package fields; stage a minimal root manifest.
COPY services-base/Cargo.toml /services-base/Cargo.toml
COPY services-base/crates/sdlc-telemetry /services-base/crates/sdlc-telemetry
COPY services-base/crates/sdlc-auth-core /services-base/crates/sdlc-auth-core
COPY services-base/crates/sdlc-shared /services-base/crates/sdlc-shared
COPY wiki/backend/Cargo.toml wiki/backend/Cargo.lock* ./
COPY wiki/backend/api/Cargo.toml api/Cargo.toml
COPY wiki/backend/app/Cargo.toml app/Cargo.toml
COPY wiki/backend/domain/Cargo.toml domain/Cargo.toml
COPY wiki/backend/infra/Cargo.toml infra/Cargo.toml
COPY wiki/backend/shared/Cargo.toml shared/Cargo.toml
COPY wiki/backend/server/Cargo.toml server/Cargo.toml
COPY wiki/backend/cli/Cargo.toml cli/Cargo.toml
COPY wiki/backend/migration/Cargo.toml migration/Cargo.toml
# Product metadata is stable; actual SDK bytes remain an independent layer input.
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} \
    for package in api app domain infra shared server cli migration; do mkdir -p "$package/src"; printf '%s\n' '' > "$package/src/lib.rs"; printf '%s\n' 'fn main() {}' > "$package/src/main.rs"; done

FROM manifests AS unit-dependencies
ARG PDLC_RUN_SLOT_SHARING=locked
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} --mount=type=cache,target=/usr/local/cargo/registry,id=${CARGO_REGISTRY_CACHE_ID}-${TARGETARCH},sharing=locked \
    cargo test --locked --release -p domain --lib --no-run \
    && cargo clean --release -p api -p app -p domain -p infra -p shared -p server -p wiki-cli -p migration

FROM unit-dependencies AS dependencies
ARG PDLC_RUN_SLOT_SHARING=locked
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} --mount=type=cache,target=/usr/local/cargo/registry,id=${CARGO_REGISTRY_CACHE_ID}-${TARGETARCH},sharing=locked \
    cargo build --locked --release -p server --bin server \
    && cargo clean --release -p api -p app -p domain -p infra -p shared -p server -p wiki-cli -p migration

FROM unit-dependencies AS unit
ARG PDLC_RUN_SLOT_SHARING=locked
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} rm -rf -- shared/src domain/src
COPY wiki/backend/shared ./shared
COPY wiki/backend/domain ./domain
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} --mount=type=cache,target=/usr/local/cargo/registry,id=${CARGO_REGISTRY_CACHE_ID}-${TARGETARCH},sharing=locked cargo test --locked --release -p domain --lib && printf 'passed\n' > /tmp/rust-unit-check

FROM dependencies AS builder
ARG PDLC_RUN_SLOT_SHARING=locked
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} --mount=type=bind,from=unit,source=/tmp/rust-unit-check,target=/tmp/rust-unit-check test "$(cat /tmp/rust-unit-check)" = passed
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} rm -rf -- api/src app/src domain/src infra/src shared/src server/src
COPY wiki/backend/api ./api
COPY wiki/backend/app ./app
COPY wiki/backend/domain ./domain
COPY wiki/backend/infra ./infra
COPY wiki/backend/shared ./shared
COPY wiki/backend/server ./server
COPY wiki/backend/migrations ./migrations
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} --mount=type=cache,target=/usr/local/cargo/registry,id=${CARGO_REGISTRY_CACHE_ID}-${TARGETARCH},sharing=locked cargo build --locked --release -p server --bin server

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
