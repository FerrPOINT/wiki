#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ENV_FILE="${PROJECT_DIR}/.env"

if [ -f "$ENV_FILE" ]; then
  # shellcheck source=/dev/null
  set -a
  # shellcheck source=/dev/null
  . "$ENV_FILE"
  set +a
fi

cd "$PROJECT_DIR"

: "${WIKI_API_URL:=http://localhost:3456/api/v1}"
: "${WIKI_BOOTSTRAP__ADMIN_EMAIL:=${WIKI_ADMIN_EMAIL:-admin@example.com}}"
: "${WIKI_BOOTSTRAP__ADMIN_PASSWORD:=${WIKI_ADMIN_PASSWORD:-}}"

if [ -z "$WIKI_BOOTSTRAP__ADMIN_PASSWORD" ]; then
  echo "ERROR: WIKI_BOOTSTRAP__ADMIN_PASSWORD is not set in .env" >&2
  exit 1
fi

echo "Logging in to ${WIKI_API_URL} as ${WIKI_BOOTSTRAP__ADMIN_EMAIL}..."
LOGIN_JSON="$(
  cargo run --quiet --manifest-path backend/Cargo.toml -p wiki-cli -- \
    --api-url "$WIKI_API_URL" \
    auth login \
    --email "$WIKI_BOOTSTRAP__ADMIN_EMAIL" \
    --password "$WIKI_BOOTSTRAP__ADMIN_PASSWORD"
)"
WIKI_TOKEN="$(printf '%s' "$LOGIN_JSON" | python3 -c 'import json, sys; print(json.load(sys.stdin)["access_token"])')"
export WIKI_TOKEN

CONTENT_FILE="$(mktemp)"
trap 'rm -f "$CONTENT_FILE"' EXIT
cat > "$CONTENT_FILE" <<'EOF'
# Демо требования Wiki

Этот документ проверяет базовый контур: создание документа, публикацию, связь с задачей, связь с фазой и evidence.

## Acceptance Criteria

- Документ опубликован.
- Task dossier видит связанный документ.
- Phase dossier видит связанный документ.
- Evidence добавлен через публичный API.
EOF

SLUG="demo-requirements-$(date +%Y%m%d%H%M%S)"
echo "Creating demo document ${SLUG}..."
DOCUMENT_JSON="$(
  cargo run --quiet --manifest-path backend/Cargo.toml -p wiki-cli -- \
    --api-url "$WIKI_API_URL" \
    --token "$WIKI_TOKEN" \
    doc create \
    --space SDLC \
    --title "Демо требования Wiki" \
    --type requirements \
    --slug "$SLUG" \
    --task SDLC-DEMO \
    --phase testing \
    --from-file "$CONTENT_FILE"
)"
DOCUMENT_ID="$(printf '%s' "$DOCUMENT_JSON" | python3 -c 'import json, sys; print(json.load(sys.stdin)["id"])')"

echo "Publishing ${DOCUMENT_ID}..."
cargo run --quiet --manifest-path backend/Cargo.toml -p wiki-cli -- \
  --api-url "$WIKI_API_URL" \
  --token "$WIKI_TOKEN" \
  doc publish "$DOCUMENT_ID" \
  --summary "Demo seed publish" >/dev/null

echo "Adding demo evidence..."
cargo run --quiet --manifest-path backend/Cargo.toml -p wiki-cli -- \
  --api-url "$WIKI_API_URL" \
  --token "$WIKI_TOKEN" \
  evidence add-link \
  --space SDLC \
  --task SDLC-DEMO \
  --phase testing \
  --title "Demo CI evidence" \
  --url "https://ci.local/jobs/wiki-demo" >/dev/null

echo "Demo data seeded."
