param(
    [string]$PostgresHost = "127.0.0.1",

    [int]$PostgresPort = 5432,

    [int]$TimeoutSeconds = 30,

    [switch]$KeepArtifacts
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
    throw "WSL is not available. Run the Docker-backed backup/restore drill on a Docker host instead."
}

$WslRepoRoot = ConvertTo-WslPath $RepoRoot
$KeepArtifactsFlag = if ($KeepArtifacts) { "1" } else { "0" }

$BashTemplate = @'
set -euo pipefail

repo_root=__REPO_ROOT__
pg_host=__POSTGRES_HOST__
pg_port=__POSTGRES_PORT__
timeout_seconds=__TIMEOUT_SECONDS__
keep_artifacts=__KEEP_ARTIFACTS__

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 is not available in WSL." >&2
    exit 1
  fi
}

require_command cargo
require_command psql
require_command pg_dump
require_command pg_restore
require_command pg_isready
require_command tar
require_command sha256sum

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

cd "$repo_root"

source_db="wiki_backup_source_$(date +%s)_$RANDOM"
restore_db="wiki_backup_restore_$(date +%s)_$RANDOM"
role="wiki_backup_$RANDOM"
pass="$(od -An -N12 -tx1 /dev/urandom | tr -d ' \n')"
tmp_root="$(mktemp -d)"
source_uploads="$tmp_root/source-uploads"
restored_uploads="$tmp_root/restored-uploads"
archive_path="$tmp_root/wiki-backup-restore-smoke.tar.gz"

cleanup() {
  su postgres -c "psql -v ON_ERROR_STOP=1 -Atc \"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname IN ('$source_db', '$restore_db') AND pid <> pg_backend_pid();\"" >/dev/null 2>&1 || true
  su postgres -c "dropdb --if-exists $restore_db" >/dev/null 2>&1 || true
  su postgres -c "dropdb --if-exists $source_db" >/dev/null 2>&1 || true
  su postgres -c "dropuser --if-exists $role" >/dev/null 2>&1 || true
  if [ "$keep_artifacts" != "1" ]; then
    if [ -n "$tmp_root" ] && [ -d "$tmp_root" ] && [[ "$tmp_root" == /tmp/* ]]; then
      rm -rf "$tmp_root"
    else
      echo "Refusing to remove unexpected temp path: $tmp_root" >&2
    fi
  else
    echo "Artifacts kept at $tmp_root"
  fi
}
trap cleanup EXIT

su postgres -c "psql -v ON_ERROR_STOP=1 -c \"CREATE ROLE $role LOGIN PASSWORD '$pass'\""
su postgres -c "createdb -O $role $source_db"
su postgres -c "createdb -O $role $restore_db"

source_url="postgres://${role}:${pass}@${pg_host}:${pg_port}/${source_db}"
restore_url="postgres://${role}:${pass}@${pg_host}:${pg_port}/${restore_db}"

echo "Applying canonical SQLx migrations to $source_db"
(
  cd "$repo_root/backend"
  DATABASE_URL="$source_url" cargo run -p migration -- up >/dev/null
)

admin_id="$(cat /proc/sys/kernel/random/uuid)"
space_id="$(cat /proc/sys/kernel/random/uuid)"
document_id="$(cat /proc/sys/kernel/random/uuid)"
revision_id="$(cat /proc/sys/kernel/random/uuid)"
attachment_id="$(cat /proc/sys/kernel/random/uuid)"
evidence_id="$(cat /proc/sys/kernel/random/uuid)"
storage_key="evidence/$evidence_id/proof.txt"

mkdir -p "$source_uploads/$(dirname "$storage_key")"
printf 'wiki backup restore smoke\nrevision=%s\n' "$revision_id" > "$source_uploads/$storage_key"
attachment_checksum="$(sha256sum "$source_uploads/$storage_key" | awk '{print $1}')"
attachment_size="$(wc -c < "$source_uploads/$storage_key" | tr -d ' ')"

psql "$source_url" -v ON_ERROR_STOP=1 <<SQL
INSERT INTO users (id, email, username, display_name, password_hash, global_role)
VALUES ('$admin_id', 'backup-smoke@example.test', 'backup-smoke', 'Backup Smoke', 'argon2-placeholder', 'admin');

INSERT INTO spaces (id, key, name, description, owner_id)
VALUES ('$space_id', 'BKP', 'Backup Smoke', 'Restore drill source data', '$admin_id');

INSERT INTO space_members (space_id, user_id, role)
VALUES ('$space_id', '$admin_id', 'admin');

INSERT INTO documents (id, space_id, slug, title, document_type, status, owner_id)
VALUES ('$document_id', '$space_id', 'restore-drill', 'Restore Drill', 'requirements', 'published', '$admin_id');

INSERT INTO document_revisions (
  id, document_id, version, title, content_markdown, content_html,
  content_text, content_checksum, summary, author_id
)
VALUES (
  '$revision_id',
  '$document_id',
  1,
  'Restore Drill',
  '# Restore Drill',
  '<h1>Restore Drill</h1>',
  'Restore Drill',
  'sha256-placeholder',
  'backup restore smoke',
  '$admin_id'
);

UPDATE documents
SET current_revision_id = '$revision_id'
WHERE id = '$document_id';

INSERT INTO document_drafts (document_id, author_id, content_markdown, base_revision_id)
VALUES ('$document_id', '$admin_id', '# Restore Drill', '$revision_id');

INSERT INTO attachments (
  id, space_id, owner_entity_type, owner_entity_id, file_name, content_type,
  size_bytes, storage_key, checksum, uploaded_by
)
VALUES (
  '$attachment_id',
  '$space_id',
  'evidence',
  '$evidence_id',
  'proof.txt',
  'text/plain',
  $attachment_size,
  '$storage_key',
  '$attachment_checksum',
  '$admin_id'
);

INSERT INTO evidence_items (
  id, space_id, document_id, evidence_type, title, attachment_id, checksum, metadata, created_by
)
VALUES (
  '$evidence_id',
  '$space_id',
  '$document_id',
  'uploaded_file',
  'Restore proof',
  '$attachment_id',
  '$attachment_checksum',
  '{"source":"backup-restore-smoke"}'::jsonb,
  '$admin_id'
);

INSERT INTO audit_log (id, actor_id, action, entity_type, entity_id, request_id)
VALUES ('00000000-0000-7000-8000-000000000001', '$admin_id', 'backup_restore.smoke', 'document', '$document_id', 'req_backup_restore_smoke');
SQL

mkdir -p "$tmp_root/backup"
echo "Creating pg_dump and attachment archive"
pg_dump "$source_url" -Fc > "$tmp_root/backup/postgres.dump"
tar -cf "$tmp_root/backup/attachments.tar" -C "$source_uploads" .
printf 'created_at=%s\nsource_db=%s\nstorage_dir=%s\n' \
  "$(date -u +%Y%m%d-%H%M%S)" "$source_db" "$source_uploads" \
  > "$tmp_root/backup/manifest.env"
tar -czf "$archive_path" -C "$tmp_root/backup" postgres.dump attachments.tar manifest.env

echo "Restoring database into $restore_db"
pg_restore --dbname "$restore_url" --clean --if-exists --no-owner < "$tmp_root/backup/postgres.dump"

mkdir -p "$restored_uploads"
tar -xf "$tmp_root/backup/attachments.tar" -C "$restored_uploads"

restored_title="$(psql "$restore_url" -Atc "SELECT title FROM document_revisions WHERE id = '$revision_id'")"
restored_evidence_checksum="$(psql "$restore_url" -Atc "SELECT checksum FROM evidence_items WHERE id = '$evidence_id'")"
restored_attachment_checksum="$(psql "$restore_url" -Atc "SELECT checksum FROM attachments WHERE id = '$attachment_id'")"
restored_file_checksum="$(sha256sum "$restored_uploads/$storage_key" | awk '{print $1}')"
restored_migration_count="$(psql "$restore_url" -Atc "SELECT count(*) FROM _sqlx_migrations WHERE success = true")"

if [ "$restored_title" != "Restore Drill" ]; then
  echo "Unexpected restored revision title: $restored_title" >&2
  exit 1
fi

if [ "$restored_evidence_checksum" != "$attachment_checksum" ] || [ "$restored_attachment_checksum" != "$attachment_checksum" ]; then
  echo "Restored DB checksum does not match source attachment checksum." >&2
  exit 1
fi

if [ "$restored_file_checksum" != "$attachment_checksum" ]; then
  echo "Restored attachment file checksum does not match source." >&2
  exit 1
fi

if [ "$restored_migration_count" -lt 1 ]; then
  echo "Restored database does not contain successful SQLx migration records." >&2
  exit 1
fi

echo "Backup/restore smoke passed: archive=$archive_path, source_db=$source_db, restore_db=$restore_db"
'@

$BashScript = $BashTemplate.
    Replace("__REPO_ROOT__", (ConvertTo-BashSingleQuotedLiteral $WslRepoRoot)).
    Replace("__POSTGRES_HOST__", (ConvertTo-BashSingleQuotedLiteral $PostgresHost)).
    Replace("__POSTGRES_PORT__", (ConvertTo-BashSingleQuotedLiteral ([string]$PostgresPort))).
    Replace("__TIMEOUT_SECONDS__", (ConvertTo-BashSingleQuotedLiteral ([string]$TimeoutSeconds))).
    Replace("__KEEP_ARTIFACTS__", (ConvertTo-BashSingleQuotedLiteral $KeepArtifactsFlag))

$EncodedScript = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($BashScript))
& wsl bash -lc "printf '%s' '$EncodedScript' | base64 -d | bash"
if ($LASTEXITCODE -ne 0) {
    throw "WSL backup/restore smoke failed with exit code $LASTEXITCODE"
}
