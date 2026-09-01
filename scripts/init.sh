#!/usr/bin/env bash
set -euo pipefail

SCRIPT_PATH="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"
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

if [ ! -f docker-compose.yml ]; then
  echo "ERROR: docker-compose.yml not found in $PROJECT_DIR" >&2
  exit 1
fi

mkdir -p traefik/letsencrypt backups

if [ ! -f .env ]; then
  echo "Creating .env from .env.example..."
  cp .env.example .env
  echo "Please edit .env before next run."
  exit 0
fi

echo "Starting PostgreSQL..."
docker compose up -d postgres

echo "Waiting for postgres healthy..."
docker compose exec -T postgres pg_isready -U "${POSTGRES_USER:-wiki}" -d "${POSTGRES_DB:-wiki}" > /dev/null

echo "Starting Wiki services..."
docker compose up -d backend frontend

echo "Waiting for backend readiness..."
docker compose exec -T backend wget -qO- http://localhost:3456/api/v1/health/ready > /dev/null

echo "Init complete."
