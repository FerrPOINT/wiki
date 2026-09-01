#!/usr/bin/env bash
set -euo pipefail

SCRIPT_PATH="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

COMPOSE_FILES=(-f docker-compose.yml)
if [ -f docker-compose.staging.yml ]; then
  COMPOSE_FILES+=(-f docker-compose.staging.yml)
fi

CURRENT_BRANCH="$(git branch --show-current)"
if [ -n "$CURRENT_BRANCH" ]; then
  git pull --ff-only origin "$CURRENT_BRANCH"
else
  git fetch --tags origin
fi

docker compose "${COMPOSE_FILES[@]}" build
docker compose "${COMPOSE_FILES[@]}" up -d postgres backend frontend
docker compose "${COMPOSE_FILES[@]}" exec -T backend wget -qO- http://localhost:3456/api/v1/health/ready > /dev/null

echo "Staging deployed."
