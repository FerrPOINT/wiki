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

: "${WIKI_BOOTSTRAP__ADMIN_EMAIL:=${WIKI_ADMIN_EMAIL:-admin@example.com}}"
: "${WIKI_BOOTSTRAP__ADMIN_PASSWORD:=${WIKI_ADMIN_PASSWORD:-}}"
: "${WIKI_BOOTSTRAP__ADMIN_USERNAME:=${WIKI_ADMIN_USERNAME:-admin}}"
: "${WIKI_BOOTSTRAP__ADMIN_DISPLAY_NAME:=${WIKI_ADMIN_DISPLAY_NAME:-Wiki Admin}}"

if [ -z "$WIKI_BOOTSTRAP__ADMIN_PASSWORD" ]; then
  echo "ERROR: WIKI_BOOTSTRAP__ADMIN_PASSWORD is not set in .env" >&2
  exit 1
fi

export WIKI_BOOTSTRAP__ADMIN_EMAIL
export WIKI_BOOTSTRAP__ADMIN_PASSWORD
export WIKI_BOOTSTRAP__ADMIN_USERNAME
export WIKI_BOOTSTRAP__ADMIN_DISPLAY_NAME

cd "$PROJECT_DIR"

echo "Ensuring admin user ${WIKI_BOOTSTRAP__ADMIN_EMAIL} through backend bootstrap..."
docker compose up -d --force-recreate backend

echo "Admin bootstrap applied. Login with ${WIKI_BOOTSTRAP__ADMIN_EMAIL} after backend healthcheck is green."
