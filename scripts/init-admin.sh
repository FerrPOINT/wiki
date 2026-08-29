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

: "${WIKI_ADMIN_EMAIL:=admin@example.com}"
: "${WIKI_ADMIN_PASSWORD:=}"

if [ -z "$WIKI_ADMIN_PASSWORD" ]; then
  echo "ERROR: WIKI_ADMIN_PASSWORD is not set in .env" >&2
  exit 1
fi

cd "$PROJECT_DIR"

echo "Creating admin user ${WIKI_ADMIN_EMAIL}..."
# Placeholder: replace with actual CLI call once backend exists
docker compose run --rm cli wiki users create-admin \
  --email "$WIKI_ADMIN_EMAIL" \
  --password "$WIKI_ADMIN_PASSWORD"

echo "Admin user created."
