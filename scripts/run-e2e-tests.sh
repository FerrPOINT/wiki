#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

COMPOSE_FILE="$REPO_ROOT/backend/docker-compose.test.yml"
export WIKI_TEST_DATABASE_URL="${WIKI_TEST_DATABASE_URL:-postgres://wiki@127.0.0.1:3458/wiki_test}"

cleanup() {
    docker compose -f "$COMPOSE_FILE" down -v
}
trap cleanup EXIT

docker compose -f "$COMPOSE_FILE" down -v
docker compose -f "$COMPOSE_FILE" up -d postgres-test

echo "Waiting for test Postgres to be healthy..."
docker compose -f "$COMPOSE_FILE" exec -T postgres-test sh -c "until pg_isready -U wiki -d wiki_test; do sleep 1; done"
echo "Running backend coverage against WIKI_TEST_DATABASE_URL=$WIKI_TEST_DATABASE_URL"

cd backend
cargo llvm-cov --workspace --json --output-path target/llvm-cov/coverage.json -- --include-ignored --test-threads=1

python3 - <<'PY'
import json, sys

# Stable-Rust realistic gate. `functions` is structurally capped below 100% by
# compiler-generated async closures and duplicate monomorphization across test
# binaries; migrations `down` and entry-point binaries are excluded upstream.
THRESHOLDS = {"lines": 77.0, "regions": 70.0, "functions": 63.0}

with open('target/llvm-cov/coverage.json') as f:
    data = json.load(f)
t = data['data'][0]['totals']
failed = False
for metric in ("lines", "regions", "functions"):
    pct = t[metric]['percent']
    ok = pct >= THRESHOLDS[metric]
    print(f"{metric:9} {pct:6.2f}% (gate >= {THRESHOLDS[metric]:.1f}%) {'OK' if ok else 'FAIL'}")
    if not ok:
        failed = True
sys.exit(1 if failed else 0)
PY
