#!/usr/bin/env bash
set -euo pipefail

SCRIPT_PATH="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

read_dotenv_value() {
  local key="$1"
  if [ ! -f .env ]; then
    return 0
  fi
  grep -E "^${key}=" .env | tail -n 1 | cut -d= -f2- | tr -d "\"'"
}

WIKI_ENVIRONMENT_VALUE="${WIKI_ENVIRONMENT:-$(read_dotenv_value WIKI_ENVIRONMENT)}"
if [ "$WIKI_ENVIRONMENT_VALUE" != "production" ]; then
  echo "Production deploy requires WIKI_ENVIRONMENT=production in the shell environment or .env." >&2
  exit 1
fi

COMPOSE_FILES=(-f docker-compose.yml)
if [ -f docker-compose.prod.yml ]; then
  COMPOSE_FILES+=(-f docker-compose.prod.yml)
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

echo "Production deployed."
