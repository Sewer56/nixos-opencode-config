<#
.SYNOPSIS
  Build OpenCode (bun install + bun run build --single), git-state cached.

.DESCRIPTION
  Builds the pinned OpenCode submodule with a git-state cache:
    - Cache key: `git -C opencode-source rev-parse HEAD`
    - Marker:    bin\.cache\opencode.sha
    - Hit when:  SHA matches AND bin\opencode.exe present AND not -ForceRebuild

  On cache hit, skips both `bun install` and `bun run build --single`.
  Without git, or if the SHA can't be resolved, cache is disabled and OpenCode
  is always rebuilt (bun install runs every time).

  build.ts emits dist\opencode-windows-x64\bin\opencode(.exe). Bun auto-appends
  .exe on Windows, but we probe both names defensively against Bun versions
  that don't.

  Native calls use $PreFlight.Bun / $PreFlight.Git for determinism.
#>
#Requires -Version 5.1

function Build-OpenCode {
  param([hashtable]$PreFlight)

  Write-PhaseHeader 'OpenCode (bun build --single)'

  if (-not $PreFlight.Bun) {
    Write-StepWarn 'skipped (bun unavailable)'
    return 'skipped'
  }

  $RepoRoot  = $script:RepoRoot
  $ocSource  = Join-Path $RepoRoot 'opencode-source'
  $ocPkgDir  = Join-Path $ocSource 'packages\opencode'
  $binDir    = Join-Path $RepoRoot 'bin'
  $cacheDir  = Join-Path $binDir '.cache'
  $shaMarker = Join-Path $cacheDir 'opencode.sha'
  $distDir   = Join-Path $ocPkgDir 'dist'
  $binDst    = Join-Path $binDir 'opencode.exe'

  # Compute git SHA for cache key (HEAD of opencode-source submodule).
  # Null-safe: a failed `git rev-parse HEAD` must degrade gracefully to
  # "cache disabled, always rebuild" - NOT throw on .Trim() of null.
  $sha = $null
  if ($PreFlight.Git -and (Test-Path (Join-Path $ocSource '.git'))) {
    Push-Location $ocSource
    try {
      $raw = & $PreFlight.Git rev-parse HEAD 2>$null
      if ($LASTEXITCODE -eq 0 -and $raw) {
        $sha = ($raw -join '').Trim()
      }
    }
    finally {
      Pop-Location
    }
    if (-not $sha) {
      Write-StepWarn 'git rev-parse HEAD failed; OpenCode cache disabled'
    }
  }

  # Cache hit?
  if (-not $script:ForceRebuild -and $sha -and (Test-Path $shaMarker) -and (Test-Path $binDst)) {
    # Get-Content -Raw returns $null on a 0-byte / truncated marker file in PS 5.1.
    # Guard against NPE on .Trim() - same null-Trim class the SHA block above fixes.
    $cached = Get-Content $shaMarker -Raw -ErrorAction SilentlyContinue
    if ($cached -and ($cached.Trim() -eq $sha)) {
      Write-StepOk "up to date (sha $sha); use -ForceRebuild to rebuild"
      return 'cached'
    }
  }

  if (-not (Test-Path $ocPkgDir)) {
    Write-StepErr "OpenCode source dir missing: $ocPkgDir"
    return 'failed'
  }

  Push-Location $ocPkgDir
  try {
    Write-Host '  bun install ...'
    & $PreFlight.Bun install 2>&1 | ForEach-Object { Write-Host "  $_" }
    if ($LASTEXITCODE -ne 0) {
      Write-StepErr 'bun install failed'
      return 'failed'
    }

    Write-Host '  bun run build --single ...'
    & $PreFlight.Bun run build --single 2>&1 | ForEach-Object { Write-Host "  $_" }
    if ($LASTEXITCODE -ne 0) {
      Write-StepErr 'bun run build --single failed'
      return 'failed'
    }
  }
  finally {
    Pop-Location
  }

  # Locate built binary. build.ts emits dist\<name>\bin\opencode; on win32 the
  # compiled binary may or may not have a .exe extension (Bun version-dependent).
  $candidates = @(
    (Join-Path $distDir 'opencode-windows-x64\bin\opencode.exe'),
    (Join-Path $distDir 'opencode-windows-x64\bin\opencode')
  )
  $src = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
  if (-not $src) {
    Write-StepErr "built binary not found under $distDir\opencode-windows-x64\bin\"
    return 'failed'
  }

  # Copy to bin\opencode.exe. Windows refuses to overwrite a held-open .exe
  # even with -Force; surface a clear message instead of a raw exception.
  try {
    Copy-Item $src $binDst -Force
  }
  catch {
    Write-StepErr "cannot overwrite $binDst - is opencode running? Close it and rerun."
    Write-Host "  $($_.Exception.Message)" -ForegroundColor DarkGray
    return 'failed'
  }
  Write-StepOk 'copied: opencode.exe'

  # Write SHA marker for next run. Non-fatal on failure (build itself succeeded).
  if ($sha) {
    try {
      Set-Content -Path $shaMarker -Value $sha -NoNewline
      Write-StepOk "wrote cache marker ($sha)"
    }
    catch {
      Write-StepWarn "could not write cache marker: $shaMarker"
      Write-Host "  $($_.Exception.Message)" -ForegroundColor DarkGray
    }
  }

  return 'built'
}
