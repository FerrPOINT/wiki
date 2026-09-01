param(
    [ValidateSet("auto", "host", "wsl")]
    [string]$CargoMode = "auto",

    [int]$TimeoutSeconds = 60,

    [switch]$KeepContainers
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$BackendDir = Join-Path $RepoRoot "backend"
$ComposeFile = Join-Path $BackendDir "docker-compose.test.yml"
$DatabaseUrl = "postgres://wiki@127.0.0.1:3458/wiki_test"

function Invoke-Checked {
    param(
        [string]$FilePath,
        [string[]]$Arguments,
        [string]$Label
    )

    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

function Test-DockerDaemon {
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        throw "Docker CLI is not available. Install Docker Desktop or run the Postgres tests against an existing database by setting WIKI_TEST_DATABASE_URL manually."
    }

    $job = Start-Job -ScriptBlock {
        $output = docker info --format "{{.ServerVersion}}" 2>&1
        [pscustomobject]@{
            ExitCode = $LASTEXITCODE
            Output = ($output -join "`n")
        }
    }

    try {
        if (-not (Wait-Job $job -Timeout 20)) {
            Stop-Job $job -ErrorAction SilentlyContinue
            throw "Docker daemon did not answer within 20 seconds. Start Docker Desktop and rerun scripts/postgres-smoke.ps1."
        }

        $result = Receive-Job $job
        if ($result.ExitCode -ne 0) {
            throw "Docker daemon is not available: $($result.Output)"
        }
    }
    finally {
        Remove-Job $job -Force -ErrorAction SilentlyContinue
    }
}

function Wait-PostgresReady {
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)

    while ((Get-Date) -lt $deadline) {
        & docker compose -f $ComposeFile exec -T postgres-test pg_isready -U wiki -d wiki_test *> $null
        if ($LASTEXITCODE -eq 0) {
            return
        }
        Start-Sleep -Seconds 1
    }

    & docker compose -f $ComposeFile logs --tail 80 postgres-test
    throw "Postgres test container was not ready within $TimeoutSeconds seconds."
}

function Resolve-CargoMode {
    if ($CargoMode -ne "auto") {
        return $CargoMode
    }

    if (Get-Command wsl -ErrorAction SilentlyContinue) {
        $wslRepoRoot = (& wsl wslpath -a $RepoRoot 2>$null).Trim()
        if ($LASTEXITCODE -eq 0 -and $wslRepoRoot) {
            & wsl bash -lc "command -v cargo >/dev/null && test -d '$wslRepoRoot/backend'" *> $null
            if ($LASTEXITCODE -eq 0) {
                return "wsl"
            }
        }
    }

    return "host"
}

function Invoke-PostgresCargoTests {
    param([string]$Mode)

    if ($Mode -eq "wsl") {
        $wslRepoRoot = (& wsl wslpath -a $RepoRoot).Trim()
        if ($LASTEXITCODE -ne 0 -or -not $wslRepoRoot) {
            throw "Could not resolve repository path inside WSL."
        }

        $command = "cd '$wslRepoRoot/backend' && WIKI_TEST_DATABASE_URL='$DatabaseUrl' cargo test -p api wiki_postgres_ -- --test-threads=1 --nocapture"
        Invoke-Checked "wsl" @("bash", "-lc", $command) "WSL cargo postgres smoke"
        return
    }

    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "Cargo is not available on the host. Install Rust, or rerun with -CargoMode wsl when WSL Rust is configured."
    }

    Push-Location $BackendDir
    $oldDatabaseUrl = $env:WIKI_TEST_DATABASE_URL
    try {
        $env:WIKI_TEST_DATABASE_URL = $DatabaseUrl
        Invoke-Checked "cargo" @("test", "-p", "api", "wiki_postgres_", "--", "--test-threads=1", "--nocapture") "Host cargo postgres smoke"
    }
    finally {
        if ($null -eq $oldDatabaseUrl) {
            Remove-Item Env:\WIKI_TEST_DATABASE_URL -ErrorAction SilentlyContinue
        }
        else {
            $env:WIKI_TEST_DATABASE_URL = $oldDatabaseUrl
        }
        Pop-Location
    }
}

if (-not (Test-Path $ComposeFile)) {
    throw "Missing compose file: $ComposeFile"
}

Test-DockerDaemon
$resolvedCargoMode = Resolve-CargoMode
Write-Host "Starting Postgres smoke with CargoMode=$resolvedCargoMode and WIKI_TEST_DATABASE_URL=$DatabaseUrl"

try {
    Invoke-Checked "docker" @("compose", "-f", $ComposeFile, "up", "-d", "postgres-test") "Docker compose up"
    Wait-PostgresReady
    Invoke-PostgresCargoTests $resolvedCargoMode
}
finally {
    if (-not $KeepContainers) {
        & docker compose -f $ComposeFile down -v
    }
}
