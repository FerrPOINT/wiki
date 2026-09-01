#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ENV_FILE="${PROJECT_DIR}/.env"
BACKUP_DIR="${PROJECT_DIR}/backups"

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
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BACKUP_NAME="wiki-${TIMESTAMP}"
BACKUP_ARCHIVE="${BACKUP_DIR}/${BACKUP_NAME}.tar.gz"

mkdir -p "$BACKUP_DIR"
TMP_DIR="$(mktemp -d "${BACKUP_DIR}/.${BACKUP_NAME}.XXXXXX")"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

write_empty_attachments_archive() {
  mkdir -p "${TMP_DIR}/empty-attachments"
  tar -cf "${TMP_DIR}/attachments.tar" -C "${TMP_DIR}/empty-attachments" .
}

cd "$PROJECT_DIR"

echo "Backing up database..."
docker compose exec -T postgres pg_dump \
  -U "$DB_USER" \
  -d "$DB_NAME" \
  -Fc \
  > "${TMP_DIR}/postgres.dump"

echo "Backing up attachments..."
if BACKEND_CONTAINER="$(docker compose ps -q backend 2>/dev/null)" && [ -n "$BACKEND_CONTAINER" ]; then
  mkdir -p "${TMP_DIR}/attachments"
  if docker compose cp "backend:${CONTAINER_STORAGE_DIR}/." "${TMP_DIR}/attachments" >/dev/null 2>&1; then
    tar -cf "${TMP_DIR}/attachments.tar" -C "${TMP_DIR}/attachments" .
  else
    echo "WARN: could not copy attachments from backend:${CONTAINER_STORAGE_DIR}; storing empty attachment archive." >&2
    write_empty_attachments_archive
  fi
else
  echo "WARN: backend container is not running; storing empty attachment archive." >&2
  write_empty_attachments_archive
fi

printf 'created_at=%s\ndb_name=%s\ndb_user=%s\nstorage_dir=%s\n' \
  "$TIMESTAMP" "$DB_NAME" "$DB_USER" "$CONTAINER_STORAGE_DIR" \
  > "${TMP_DIR}/manifest.env"

echo "Creating archive..."
tar -czf "$BACKUP_ARCHIVE" -C "$TMP_DIR" \
  postgres.dump \
  attachments.tar \
  manifest.env

echo "Backup created: ${BACKUP_ARCHIVE}"
