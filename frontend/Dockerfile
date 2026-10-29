# syntax=docker/dockerfile:1.27.1@sha256:4edf897a3ffa55b89f906fc8cc78afdb3f1834cc9c7083565e611a8a7d5fe99e
# Umbrella build for PDLC root Compose (context = the parent source workspace).
# The repo's own frontend/Dockerfile builds with context=repo-root only; this
# variant additionally stages the sibling services-base frontend so the
# `@pdlc/ui` file dependency (../../services-base/base-modules/ui) resolves.
FROM node:22.22.2-bookworm-slim AS toolchain
ARG TARGETARCH
ARG PNPM_CACHE_ID=pdlc-operator-downloads-v1-pnpm10-node22

RUN corepack enable && corepack prepare pnpm@10.28.1 --activate

# Preserve the workspace sibling layout so both pnpm's file dependency and
# Tailwind's shared UI source path resolve the same way as a local checkout.
WORKDIR /workspace/wiki/frontend

# Product scripts are generation inputs, not dependency installation inputs.
FROM toolchain AS install-manifest
COPY wiki/frontend/package.json ./
RUN node -e 'const fs = require("node:fs"); const pkg = JSON.parse(fs.readFileSync("package.json", "utf8")); delete pkg.scripts; fs.writeFileSync("package.json", JSON.stringify(pkg) + "\n");'

FROM toolchain AS dependencies
COPY --from=install-manifest /workspace/wiki/frontend/package.json ./
COPY wiki/frontend/pnpm-lock.yaml wiki/frontend/pnpm-workspace.yaml ./
COPY services-base/base-modules/ui /workspace/services-base/base-modules/ui
# Keep dependency lifecycle scripts (including native tooling) enabled.
RUN --mount=type=cache,id=${PNPM_CACHE_ID}-${TARGETARCH},target=/pnpm/store,sharing=locked ln -s /workspace/services-base /workspace/wiki/services-base && pnpm install --frozen-lockfile --store-dir=/pnpm/store --side-effects-cache=false --verify-store-integrity=true

FROM dependencies AS generated
COPY wiki/frontend/package.json ./
COPY wiki/openapi/openapi.json /workspace/wiki/openapi/openapi.json
COPY wiki/frontend/scripts/generate-exports.mjs ./scripts/generate-exports.mjs
COPY wiki/frontend/.prettierrc ./.prettierrc
COPY wiki/frontend/scripts/base-generate-openapi.mjs ./scripts/base-generate-openapi.mjs
RUN pnpm run postinstall && test -s src/api/generated.ts && test -s src/api/generated-exports.ts

FROM dependencies AS unit
COPY wiki/frontend/ ./
COPY wiki/openapi/openapi.json /workspace/wiki/openapi/openapi.json
COPY --from=generated /workspace/wiki/frontend/src/api/generated.ts ./src/api/generated.ts
COPY --from=generated /workspace/wiki/frontend/src/api/generated-exports.ts ./src/api/generated-exports.ts
ARG VITE_NAMESPACE_ENABLED="false"
ENV VITE_NAMESPACE_ENABLED=${VITE_NAMESPACE_ENABLED}
ARG VITE_PLATFORM_BRANDING_URL=""
ENV VITE_PLATFORM_BRANDING_URL=${VITE_PLATFORM_BRANDING_URL}
ARG VITE_PLATFORM_SERVICES_URL=""
ENV VITE_PLATFORM_SERVICES_URL=${VITE_PLATFORM_SERVICES_URL}
ARG VITE_AUTH_ISSUER="http://localhost:7701"
ENV VITE_AUTH_ISSUER=${VITE_AUTH_ISSUER}
ARG VITE_AUTH_NAMESPACE="sdlc"
ENV VITE_AUTH_NAMESPACE=${VITE_AUTH_NAMESPACE}
ARG VITE_PLATFORM_ORIGIN=""
ENV VITE_PLATFORM_ORIGIN=${VITE_PLATFORM_ORIGIN}
RUN pnpm run test:unit && pnpm exec tsc --noEmit && printf 'passed\n' > /tmp/frontend-unit-check

FROM dependencies AS builder
COPY --exclude=src/**/*.test.ts --exclude=src/**/*.test.tsx --exclude=src/**/*.spec.ts --exclude=src/**/*.spec.tsx --exclude=src/**/*.test.mjs --exclude=src/**/*.test.cjs --exclude=src/*.test.ts --exclude=src/*.test.tsx --exclude=src/*.spec.ts --exclude=src/*.spec.tsx --exclude=src/*.test.mjs --exclude=src/*.test.cjs --exclude=src/test/** --exclude=vitest.config.ts --exclude=e2e/** --exclude=scripts/tailwind-source-inputs.test.mjs --exclude=scripts/tests/generate-exports.test.mjs --exclude=Dockerfile* --exclude=nginx.conf --exclude=scripts/tests/generate-exports-prepared.test.mjs wiki/frontend/ ./
COPY wiki/openapi/openapi.json /workspace/wiki/openapi/openapi.json
COPY --from=generated /workspace/wiki/frontend/src/api/generated.ts ./src/api/generated.ts
COPY --from=generated /workspace/wiki/frontend/src/api/generated-exports.ts ./src/api/generated-exports.ts
ARG VITE_NAMESPACE_ENABLED="false"
ENV VITE_NAMESPACE_ENABLED=${VITE_NAMESPACE_ENABLED}
ARG VITE_PLATFORM_BRANDING_URL=""
ENV VITE_PLATFORM_BRANDING_URL=${VITE_PLATFORM_BRANDING_URL}
ARG VITE_PLATFORM_SERVICES_URL=""
ENV VITE_PLATFORM_SERVICES_URL=${VITE_PLATFORM_SERVICES_URL}
ARG VITE_AUTH_ISSUER="http://localhost:7701"
ENV VITE_AUTH_ISSUER=${VITE_AUTH_ISSUER}
ARG VITE_AUTH_NAMESPACE="sdlc"
ENV VITE_AUTH_NAMESPACE=${VITE_AUTH_NAMESPACE}
ARG VITE_PLATFORM_ORIGIN=""
ENV VITE_PLATFORM_ORIGIN=${VITE_PLATFORM_ORIGIN}
RUN --mount=type=bind,from=unit,source=/tmp/frontend-unit-check,target=/tmp/frontend-unit-check test "$(cat /tmp/frontend-unit-check)" = passed
RUN pnpm run build:compile

FROM nginx:alpine
ARG VITE_NAMESPACE_ENABLED="false"
LABEL io.pdlc.namespace-ui.enabled=${VITE_NAMESPACE_ENABLED}
COPY --from=builder /workspace/wiki/frontend/dist /usr/share/nginx/html
COPY wiki/frontend/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
