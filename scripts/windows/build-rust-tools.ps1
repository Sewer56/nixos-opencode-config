<#
.SYNOPSIS
  Build Rust tools: workspace members + rust-llm-tidy submodule.

.DESCRIPTION
  Two phases:
    - Main workspace: cargo build --release --workspace in tools\
      (matches flake.nix cargoBuildFlags). Copies 5 .exe to bin\:
        opencode-model-switcher, opencode-sessions,
        chunk-files-by-tokens, token-count-after-expand,
        iterate-static-check
    - rust-llm-tidy submodule (separate workspace at
      tools\rust-llm-tidy\src\Cargo.toml). Auto-runs
      `git submodule update --init` if the submodule is missing.
      Copies rust-llm-tidy.exe (the upstream [[bin]] name from
      rust-llm-tidy-cli) to bin\.

  Native calls use the resolved paths from the pre-flight table
  ($PreFlight.Cargo / .Git) rather than bare names, so the same binary
  detected up front is the one invoked.
#>
#Requires -Version 5.1

# Build the main Rust workspace (5 tool binaries).
function Build-RustWorkspace {
  param([hashtable]$PreFlight)

  Write-PhaseHeader 'Rust tools (workspace)'

  if (-not $PreFlight.Cargo) {
    Write-StepWarn 'skipped (cargo unavailable)'
    return 'skipped'
  }

  $RepoRoot = $script:RepoRoot
  $toolsDir = Join-Path $RepoRoot 'tools'
  $binDir   = Join-Path $RepoRoot 'bin'

  if (-not (Test-Path (Join-Path $toolsDir 'Cargo.toml'))) {
    Write-StepErr "tools\Cargo.toml missing: $toolsDir"
    return 'failed'
  }

  Push-Location $toolsDir
  try {
    Write-Host '  cargo build --release --workspace ...'
    & $PreFlight.Cargo build --release --workspace 2>&1 | ForEach-Object { Write-Host "  $_" }
    if ($LASTEXITCODE -ne 0) {
      Write-StepErr 'cargo build --release --workspace failed'
      return 'failed'
    }
  }
  finally {
    Pop-Location
  }

  # Copy each workspace member's binary. Order matches tools/Cargo.toml
  # member list (rust-llm-tidy is handled separately below).
  $binaries = @(
    'opencode-model-switcher',
    'opencode-sessions',
    'chunk-files-by-tokens',
    'token-count-after-expand',
    'iterate-static-check'
  )

  $result = 'done'
  foreach ($name in $binaries) {
    $src = Join-Path $toolsDir "target\release\$name.exe"
    $dst = Join-Path $binDir "$name.exe"
    if (-not (Copy-FileIfNewer $src $dst -Force:$script:ForceRebuild)) {
      $result = 'failed'
    }
  }
  return $result
}

# Build the rust-llm-tidy submodule (separate workspace).
function Build-RustLLMTidy {
  param([hashtable]$PreFlight)

  Write-PhaseHeader 'rust-llm-tidy (submodule)'

  if (-not $PreFlight.Cargo) {
    Write-StepWarn 'skipped (cargo unavailable)'
    return 'skipped'
  }

  $RepoRoot      = $script:RepoRoot
  # Forward slash in the submodule pathspec - git matches it against the
  # .gitmodules path regardless of platform; backslash works on Windows but
  # is non-canonical and breaks under WSL.
  $submodulePath = 'tools/rust-llm-tidy'
  $submoduleAbs  = Join-Path $RepoRoot $submodulePath
  $srcWorkspace  = Join-Path $submoduleAbs 'src'
  $cliCargoPath  = Join-Path $srcWorkspace 'Cargo.toml'
  $binDir        = Join-Path $RepoRoot 'bin'

  # Auto-init submodule if Cargo.toml is missing
  if (-not (Test-Path $cliCargoPath)) {
    if (-not $PreFlight.Git) {
      Write-StepErr 'rust-llm-tidy submodule missing and git unavailable (cannot init)'
      return 'failed'
    }
    Write-StepWarn "submodule not initialized; running: git submodule update --init $submodulePath"
    Push-Location $RepoRoot
    try {
      & $PreFlight.Git submodule update --init $submodulePath 2>&1 | ForEach-Object { Write-Host "  $_" }
      if ($LASTEXITCODE -ne 0) {
        Write-StepErr 'git submodule update --init failed'
        return 'failed'
      }
    }
    finally {
      Pop-Location
    }
    if (-not (Test-Path $cliCargoPath)) {
      Write-StepErr "submodule init succeeded but Cargo.toml still missing: $cliCargoPath"
      return 'failed'
    }
  }

  Write-Host '  cargo build --release (rust-llm-tidy workspace) ...'
  & $PreFlight.Cargo build --release --manifest-path $cliCargoPath 2>&1 | ForEach-Object { Write-Host "  $_" }
  if ($LASTEXITCODE -ne 0) {
    Write-StepErr 'cargo build for rust-llm-tidy failed'
    return 'failed'
  }

  # cli/Cargo.toml [[bin]] name = "rust-llm-tidy"; cargo emits that name.
  $src = Join-Path $srcWorkspace "target\release\rust-llm-tidy.exe"
  $dst = Join-Path $binDir 'rust-llm-tidy.exe'
  if (-not (Copy-FileIfNewer $src $dst -Force:$script:ForceRebuild)) {
    return 'failed'
  }
  return 'done'
}
