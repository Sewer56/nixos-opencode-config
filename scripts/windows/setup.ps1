<#
.SYNOPSIS
  Windows one-shot setup for the opencode config repo.

.DESCRIPTION
  Orchestrates junctions, secrets, env, Rust tool builds, and OpenCode
  build (git-state cached). Dot-sources phase modules from scripts/windows/.

  Phases:
    1. Pre-flight        - detect cargo / bun / git (warn, never fatal)
    2. Filesystem        - junctions, secrets dir, OPENCODE_ENABLE_EXA, bin/.cache
    3. Build rust tools   - cargo build --release --workspace + rust-llm-tidy submodule
    4. Build OpenCode    - bun install + bun run build --single (cached by git SHA)
    5. PATH              - append ~\opencode\bin to User PATH if missing
    6. Summary           - phase results table

  Missing cargo/bun -> affected phases SKIPPED with WARN (not fatal).
  Missing git       -> OpenCode cache disabled (always rebuilds).
  Missing the rust-llm-tidy submodule -> auto `git submodule update --init`.

  Built binaries land in <repo>\bin\ (gitignored), reachable as
  ~\opencode\bin via the repo junction.

.PARAMETER RepoRoot
  Repo root path. Defaults to <repo> inferred from this script's location.

.PARAMETER UseEnvVar
  Persist OPENCODE_CONFIG_DIR (User) instead of creating the config junction.

.PARAMETER ForceRebuild
  Bypass the OpenCode git-SHA cache; force-copy all Rust .exe even if mtime stale.

.PARAMETER SkipRustTools
.PARAMETER SkipRustLLMTidy
.PARAMETER SkipOpenCode
  Skip individual build phases.

.EXAMPLE
  pwsh ./scripts/windows/setup.ps1
  pwsh ./scripts/windows/setup.ps1 -ForceRebuild
  pwsh ./scripts/windows/setup.ps1 -UseEnvVar -SkipOpenCode
#>
#Requires -Version 5.1
[CmdletBinding()]
param(
  [string]$RepoRoot,
  [switch]$UseEnvVar,
  [switch]$ForceRebuild,
  [switch]$SkipRustTools,
  [switch]$SkipRustLLMTidy,
  [switch]$SkipOpenCode
)

$ErrorActionPreference = 'Stop'

# Resolve repo root: $PSScriptRoot is <repo>/scripts/windows.
if (-not $RepoRoot) {
  $RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}

# Shared state consumed by dot-sourced modules ($script: scope). Functions in
# the modules read these at *call* time, so they resolve to this script scope.
$script:RepoRoot     = $RepoRoot
$script:UseEnvVar    = [bool]$UseEnvVar
$script:ForceRebuild = [bool]$ForceRebuild

$scriptsDir = Join-Path $RepoRoot 'scripts\windows'

# Dot-source phase modules. Order matters: common first (defines helpers the
# others call), then prerequisites (returns the tool table), then the phase
# modules that consume it.
. (Join-Path $scriptsDir 'common.ps1')
. (Join-Path $scriptsDir 'prerequisites.ps1')
. (Join-Path $scriptsDir 'setup-filesystem.ps1')
. (Join-Path $scriptsDir 'build-rust-tools.ps1')
. (Join-Path $scriptsDir 'build-opencode.ps1')

# --- phases ------------------------------------------------------------------
$results = [ordered]@{}

$pre = Invoke-PreFlight
$results['Pre-flight'] = 'done'

Invoke-FilesystemSetup
$results['Filesystem'] = 'done'

if (-not $SkipRustTools) {
  $results['Rust workspace'] = Build-RustWorkspace $pre
}
else {
  Write-PhaseHeader 'Rust tools (workspace)'
  Write-StepWarn 'skipped (-SkipRustTools)'
  $results['Rust workspace'] = 'skipped'
}

if (-not $SkipRustLLMTidy) {
  $results['rust-llm-tidy'] = Build-RustLLMTidy $pre
}
else {
  Write-PhaseHeader 'rust-llm-tidy (submodule)'
  Write-StepWarn 'skipped (-SkipRustLLMTidy)'
  $results['rust-llm-tidy'] = 'skipped'
}

if (-not $SkipOpenCode) {
  $results['OpenCode'] = Build-OpenCode $pre
}
else {
  Write-PhaseHeader 'OpenCode (bun build --single)'
  Write-StepWarn 'skipped (-SkipOpenCode)'
  $results['OpenCode'] = 'skipped'
}

# PATH: append ~\opencode\bin (== $RepoRoot\bin via junction) to User PATH.
$binDir = Join-Path $RepoRoot 'bin'
Add-ToUserPath $binDir
$results['PATH'] = 'done'

# --- summary -----------------------------------------------------------------
Write-Host ''
Write-Host '=== Summary ===' -ForegroundColor Cyan
foreach ($k in $results.Keys) {
  $status = $results[$k]
  $color = switch ($status) {
    'done'    { 'Green' }
    'cached'  { 'DarkGray' }
    'built'   { 'Green' }
    'skipped' { 'Yellow' }
    'failed'  { 'Red' }
    default   { 'White' }
  }
  Write-Host ("  {0,-22} {1}" -f $k, $status) -ForegroundColor $color
}
Write-Host ''
Write-Host 'Done.' -ForegroundColor Green
