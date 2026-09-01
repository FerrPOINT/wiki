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

: "${WIKI_DB_USER:=wiki}"
: "${WIKI_DB_NAME:=wiki}"

DB_USER="${POSTGRES_USER:-$WIKI_DB_USER}"
DB_NAME="${POSTGRES_DB:-$WIKI_DB_NAME}"
CONTAINER_STORAGE_DIR="${WIKI_STORAGE__DIR:-/var/lib/wiki/uploads}"

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <backup.tar.gz>" >&2
  exit 1
fi

BACKUP_ARCHIVE="$1"
TMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

if [ ! -f "$BACKUP_ARCHIVE" ]; then
  echo "ERROR: backup archive not found: $BACKUP_ARCHIVE" >&2
  exit 1
fi

cd "$PROJECT_DIR"

echo "Extracting backup..."
tar -xzf "$BACKUP_ARCHIVE" -C "$TMP_DIR"

if [ ! -f "${TMP_DIR}/postgres.dump" ]; then
  echo "ERROR: backup archive does not contain postgres.dump" >&2
  exit 1
fi

if [ ! -f "${TMP_DIR}/attachments.tar" ]; then
  echo "ERROR: backup archive does not contain attachments.tar" >&2
  exit 1
fi

echo "Stopping application services..."
docker compose stop backend frontend >/dev/null 2>&1 || true

echo "Ensuring PostgreSQL is running..."
docker compose up -d postgres >/dev/null

echo "Restoring database..."
docker compose exec -T postgres pg_restore \
  -U "$DB_USER" \
  -d "$DB_NAME" \
  --clean --if-exists --no-owner \
  < "${TMP_DIR}/postgres.dump"

echo "Restoring attachments..."
mkdir -p "${TMP_DIR}/attachments"
tar -xf "${TMP_DIR}/attachments.tar" -C "${TMP_DIR}/attachments"

if [ -z "$(docker compose ps -a -q backend 2>/dev/null)" ]; then
  docker compose create backend >/dev/null
fi

docker compose run --rm --no-deps --entrypoint sh backend -c \
  'dir="${WIKI_STORAGE__DIR:-/var/lib/wiki/uploads}"; mkdir -p "$dir"; rm -rf "$dir"/* "$dir"/.[!.]* "$dir"/..?*'
docker compose cp "${TMP_DIR}/attachments/." "backend:${CONTAINER_STORAGE_DIR}" >/dev/null

echo "Restore complete."
echo "Start application services with: docker compose up -d backend frontend"
