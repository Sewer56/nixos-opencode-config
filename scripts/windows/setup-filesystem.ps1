<#
.SYNOPSIS
  Filesystem setup: junctions, secrets dir, env vars, bin/ and bin/.cache/ dirs.

.DESCRIPTION
  Replaces the legacy bootstrap-windows.ps1. Folds into the orchestrator:
    - ~\opencode          -> repo root        (junction)
    - ~\.config\opencode  -> repo \config\    (junction, or env var if -UseEnvVar)
    - ~\.secrets\         (created if missing; expected files checked)
    - OPENCODE_ENABLE_EXA=1 (User scope)      (Linux HM wrapper sets it per-invocation)
    - repo\bin\           (created if missing)
    - repo\bin\.cache\    (created if missing; holds OpenCode SHA marker)
#>
#Requires -Version 5.1

function Invoke-FilesystemSetup {
  Write-PhaseHeader 'Junctions, secrets, environment'

  $home_ = $env:USERPROFILE
  if (-not $home_) { throw 'USERPROFILE not set; cannot locate user home.' }

  $RepoRoot      = $script:RepoRoot
  $ConfigTarget  = Join-Path $RepoRoot 'config'
  $ShortcutLink  = Join-Path $home_    'opencode'
  $ConfigLink    = Join-Path $home_    '.config\opencode'

  if (-not (Test-Path $RepoRoot -PathType Container)) {
    throw "RepoRoot does not exist: $RepoRoot"
  }
  if (-not (Test-Path $ConfigTarget -PathType Container)) {
    throw "config dir missing in repo: $ConfigTarget"
  }

  # repo shortcut junction: ~\opencode -> repo root
  Set-Junction -Link $ShortcutLink -Target $RepoRoot -Label 'repo shortcut (~\opencode)'

  # config dir: junction or env var (OpenCode honours OPENCODE_CONFIG_DIR over the default)
  if ($script:UseEnvVar) {
    Set-UserEnvVar 'OPENCODE_CONFIG_DIR' $ConfigTarget
  }
  else {
    Set-Junction -Link $ConfigLink -Target $ConfigTarget -Label 'opencode config (~\.config\opencode)'
  }

  # secrets dir + expected-file check.
  # opencode.json references these via {file:~/.secrets/<name>}; missing file =>
  # OpenCode InvalidError at load. Create the dir; warn about absent files.
  $SecretsDir = Join-Path $home_ '.secrets'
  if (-not (Test-Path $SecretsDir)) {
    New-Item -ItemType Directory -Path $SecretsDir | Out-Null
    Write-StepOk "created secrets dir: $SecretsDir"
  }
  else {
    Write-StepOk "secrets dir exists: $SecretsDir"
  }

  $expected = 'axonhub-url', 'axonhub-key', 'axonhub-work-key', 'github-token'
  $missing = $expected | Where-Object { -not (Test-Path (Join-Path $SecretsDir $_)) }
  if ($missing) {
    Write-StepWarn ("missing secret file(s): {0}" -f ($missing -join ', '))
    Write-Host '         OpenCode will refuse to load until these exist.' -ForegroundColor Yellow
  }
  else {
    Write-StepOk 'all expected secret files present'
  }

  # OPENCODE_ENABLE_EXA=1 - Linux HM wrapper sets this per-invocation; on Windows
  # we persist it once in the User environment so the bare opencode.exe picks it up.
  Set-UserEnvVar 'OPENCODE_ENABLE_EXA' '1'

  # bin dir + cache subdir
  $binDir = Join-Path $RepoRoot 'bin'
  if (-not (Test-Path $binDir)) {
    New-Item -ItemType Directory -Path $binDir | Out-Null
    Write-StepOk "created bin dir: $binDir"
  }
  else {
    Write-StepOk "bin dir exists: $binDir"
  }

  $cacheDir = Join-Path $binDir '.cache'
  if (-not (Test-Path $cacheDir)) {
    New-Item -ItemType Directory -Path $cacheDir | Out-Null
    Write-StepOk "created cache dir: $cacheDir"
  }
}
