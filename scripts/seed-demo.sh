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

echo "Seeding demo data..."
# Placeholder: replace with actual CLI call once Wiki seed command exists.
docker compose run --rm cli wiki seed demo \
  --spaces 2 \
  --documents-per-space 10 \
  --task-dossiers 5

echo "Demo data seeded."
