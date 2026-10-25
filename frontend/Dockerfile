# syntax=docker/dockerfile:1.27.1@sha256:4edf897a3ffa55b89f906fc8cc78afdb3f1834cc9c7083565e611a8a7d5fe99e
# Umbrella build for the SDLC fleet root compose (context = /opt/dev/sdlc).
# The repo's own frontend/Dockerfile builds with context=repo-root only; this
# variant additionally stages the sibling services-base frontend so the
# `@sdlc/ui` file dependency (../../services-base/frontend) resolves.
FROM node:22.20.0-bookworm-slim AS toolchain
ARG PDLC_RUN_SLOT_SHARING=locked
ARG TARGETARCH
ARG PNPM_CACHE_ID=pdlc-operator-downloads-v1-pnpm10-node22

RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} corepack enable && corepack prepare pnpm@10.28.1 --activate

# Preserve the workspace sibling layout so both pnpm's file dependency and
# Tailwind's shared UI source path resolve the same way as a local checkout.
WORKDIR /workspace/wiki/frontend

FROM toolchain AS install-manifest
ARG PDLC_RUN_SLOT_SHARING=locked
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} --mount=type=bind,source=wiki/frontend/package.json,target=/tmp/product-package.json node -e 'const fs = require("node:fs"); const pkg = JSON.parse(fs.readFileSync("/tmp/product-package.json", "utf8")); if (!pkg || Array.isArray(pkg) || typeof pkg !== "object") throw new Error("Package manifest must be an object"); delete pkg.scripts; fs.writeFileSync("package.json", JSON.stringify(pkg) + "\n");'

FROM toolchain AS dependencies
ARG PDLC_RUN_SLOT_SHARING=locked
COPY --from=install-manifest /workspace/wiki/frontend/package.json ./
COPY wiki/frontend/pnpm-lock.yaml wiki/frontend/pnpm-workspace.yaml ./
COPY services-base/frontend /workspace/services-base/frontend
# pnpm resolves file: dependencies from the importer package root.
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} --mount=type=cache,id=${PNPM_CACHE_ID}-${TARGETARCH},target=/pnpm/store,sharing=locked ln -s /workspace/services-base /workspace/wiki/services-base && pnpm install --store-dir=/pnpm/store --frozen-lockfile --ignore-scripts


FROM dependencies AS generated
ARG PDLC_RUN_SLOT_SHARING=locked
COPY wiki/frontend/package.json ./
COPY wiki/openapi/openapi.json /workspace/wiki/openapi/openapi.json
COPY wiki/frontend/scripts/generate-exports.mjs ./scripts/generate-exports.mjs
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} pnpm run postinstall && test -s src/api/generated.ts && test -s src/api/generated-exports.ts

FROM dependencies AS unit
ARG PDLC_RUN_SLOT_SHARING=locked
COPY wiki/frontend/nginx.conf ./nginx.conf
COPY --exclude=Dockerfile* --exclude=**/Dockerfile* --exclude=.dockerignore --exclude=**/.dockerignore --exclude=nginx.conf --exclude=**/nginx.conf wiki/frontend/ ./
COPY --from=generated /workspace/wiki/frontend/src/api/generated.ts ./src/api/generated.ts
COPY --from=generated /workspace/wiki/frontend/src/api/generated-exports.ts ./src/api/generated-exports.ts
ARG VITE_PLATFORM_BRANDING_URL=""
ENV VITE_PLATFORM_BRANDING_URL=${VITE_PLATFORM_BRANDING_URL}
ARG VITE_PLATFORM_SERVICES_URL=""
ENV VITE_PLATFORM_SERVICES_URL=${VITE_PLATFORM_SERVICES_URL}
ARG VITE_AUTH_ISSUER="http://localhost:7701"
ENV VITE_AUTH_ISSUER=${VITE_AUTH_ISSUER}
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} pnpm test:unit && pnpm exec tsc --noEmit && printf 'passed\n' > /tmp/frontend-unit-check

FROM dependencies AS builder
ARG PDLC_RUN_SLOT_SHARING=locked
COPY --exclude=Dockerfile* --exclude=**/Dockerfile* --exclude=.dockerignore --exclude=**/.dockerignore --exclude=nginx.conf --exclude=**/nginx.conf --exclude=**/*.test.* --exclude=**/*.spec.* --exclude=**/__tests__/** --exclude=vitest.config.ts --exclude=e2e/** --exclude=playwright.config.ts --exclude=src/test/** wiki/frontend/ ./
COPY --from=generated /workspace/wiki/frontend/src/api/generated.ts ./src/api/generated.ts
COPY --from=generated /workspace/wiki/frontend/src/api/generated-exports.ts ./src/api/generated-exports.ts
# A changed unit file reruns checks; identical production inputs keep compilation cached.
ARG VITE_PLATFORM_BRANDING_URL=""
ENV VITE_PLATFORM_BRANDING_URL=${VITE_PLATFORM_BRANDING_URL}
ARG VITE_PLATFORM_SERVICES_URL=""
ENV VITE_PLATFORM_SERVICES_URL=${VITE_PLATFORM_SERVICES_URL}
ARG VITE_AUTH_ISSUER="http://localhost:7701"
ENV VITE_AUTH_ISSUER=${VITE_AUTH_ISSUER}
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} --mount=type=bind,from=unit,source=/tmp/frontend-unit-check,target=/tmp/frontend-unit-check test "$(cat /tmp/frontend-unit-check)" = passed
# The required unit stage already typechecks these production sources with noEmit.
RUN --mount=type=cache,id=pdlc-host-run-slot-v1,target=/.__pdlc_build_slot__,sharing=${PDLC_RUN_SLOT_SHARING} pnpm exec vite build

FROM nginx:alpine
ARG PDLC_RUN_SLOT_SHARING=locked
COPY --from=builder /workspace/wiki/frontend/dist /usr/share/nginx/html
COPY wiki/frontend/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
