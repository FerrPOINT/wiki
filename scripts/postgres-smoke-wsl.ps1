param(
    [string]$PostgresHost = "127.0.0.1",

    [int]$PostgresPort = 5432,

    [int]$TimeoutSeconds = 30
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

function ConvertTo-BashSingleQuotedLiteral {
    param([string]$Value)

    return "'" + $Value.Replace("'", "'`"`"'`"`"'") + "'"
}

function ConvertTo-WslPath {
    param([string]$Path)

    $ResolvedPath = (Resolve-Path -LiteralPath $Path).Path
    if ($ResolvedPath -match "^([A-Za-z]):\\(.*)$") {
        $Drive = $Matches[1].ToLowerInvariant()
        $Rest = $Matches[2].Replace("\", "/")
        return "/mnt/$Drive/$Rest"
    }

    $WslPath = & wsl wslpath -a $ResolvedPath 2>$null
    if ($LASTEXITCODE -ne 0 -or -not $WslPath) {
        throw "Could not resolve repository path inside WSL."
    }
    return ($WslPath | Select-Object -First 1).Trim()
}

if (-not (Get-Command wsl -ErrorAction SilentlyContinue)) {
    throw "WSL is not available. Use scripts/postgres-smoke.ps1 with Docker, or set WIKI_TEST_DATABASE_URL manually."
}

$WslRepoRoot = ConvertTo-WslPath $RepoRoot

$BashTemplate = @'
set -euo pipefail

repo_root=__REPO_ROOT__
pg_host=__POSTGRES_HOST__
pg_port=__POSTGRES_PORT__
timeout_seconds=__TIMEOUT_SECONDS__

if ! command -v cargo >/dev/null 2>&1; then
  echo "Cargo is not available in WSL." >&2
  exit 1
fi

if ! command -v psql >/dev/null 2>&1; then
  echo "psql is not available in WSL." >&2
  exit 1
fi

if ! command -v pg_isready >/dev/null 2>&1; then
  echo "pg_isready is not available in WSL." >&2
  exit 1
fi

if ! id postgres >/dev/null 2>&1; then
  echo "System user postgres is not available in WSL." >&2
  exit 1
fi

if ! su postgres -c "psql -Atc 'select 1'" >/dev/null 2>&1; then
  echo "Could not access local PostgreSQL as system user postgres." >&2
  exit 1
fi

if ! pg_isready -h "$pg_host" -p "$pg_port" -t "$timeout_seconds" >/dev/null 2>&1; then
  echo "PostgreSQL is not ready at $pg_host:$pg_port." >&2
  exit 1
fi

cd "$repo_root/backend"

db="wiki_smoke_$(date +%s)_$RANDOM"
role="wiki_smoke_$RANDOM"
pass="$(od -An -N12 -tx1 /dev/urandom | tr -d ' \n')"

cleanup() {
  su postgres -c "psql -v ON_ERROR_STOP=1 -Atc \"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$db' AND pid <> pg_backend_pid();\"" >/dev/null 2>&1 || true
  su postgres -c "dropdb --if-exists $db" >/dev/null 2>&1 || true
  su postgres -c "dropuser --if-exists $role" >/dev/null 2>&1 || true
}

trap cleanup EXIT

su postgres -c "psql -v ON_ERROR_STOP=1 -c \"CREATE ROLE $role LOGIN PASSWORD '$pass'\""
su postgres -c "createdb -O $role $db"

export WIKI_TEST_DATABASE_URL="postgres://${role}:${pass}@${pg_host}:${pg_port}/${db}"
echo "Running wiki_postgres_ tests against isolated WSL database $db as $role"
cargo test -p api wiki_postgres_ -- --test-threads=1 --nocapture
'@

$BashScript = $BashTemplate.
    Replace("__REPO_ROOT__", (ConvertTo-BashSingleQuotedLiteral $WslRepoRoot)).
    Replace("__POSTGRES_HOST__", (ConvertTo-BashSingleQuotedLiteral $PostgresHost)).
    Replace("__POSTGRES_PORT__", (ConvertTo-BashSingleQuotedLiteral ([string]$PostgresPort))).
    Replace("__TIMEOUT_SECONDS__", (ConvertTo-BashSingleQuotedLiteral ([string]$TimeoutSeconds)))

$EncodedScript = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($BashScript))
& wsl bash -lc "printf '%s' '$EncodedScript' | base64 -d | bash"
if ($LASTEXITCODE -ne 0) {
    throw "WSL PostgreSQL smoke failed with exit code $LASTEXITCODE"
}
