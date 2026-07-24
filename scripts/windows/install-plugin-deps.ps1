<#
.SYNOPSIS
  Install runtime dependencies for local OpenCode plugins.

.DESCRIPTION
  Dot-sourced by setup.ps1. Iterates config\plugins\*\ and runs
  `bun install --production` in every plugin directory whose package.json
  declares runtime dependencies. Without this, plugins with runtime deps
  (e.g. xdg-basedir) fail their dynamic import inside OpenCode *silently*
  (the loader only emits a session toast, nothing in CLI output or logs).

  Uses --frozen-lockfile when the plugin has a committed bun.lock so installs
  never rewrite submodule lockfiles and dirty the tree.

  Idempotent: bun install is a no-op when node_modules is already in sync.
#>
#Requires -Version 5.1

function Invoke-PluginDeps {
  param([hashtable]$PreFlight)

  Write-PhaseHeader 'Plugin dependencies (bun install)'

  if (-not $PreFlight.Bun) {
    # Not benign: plugins with runtime deps will silently fail to load.
    Write-StepErr 'bun unavailable; plugins with runtime deps will NOT load in OpenCode'
    return 'failed'
  }

  $pluginsDir = Join-Path $script:RepoRoot 'config\plugins'
  if (-not (Test-Path $pluginsDir -PathType Container)) {
    Write-StepWarn "no plugins dir: $pluginsDir"
    return 'skipped'
  }

  $failed = @()
  foreach ($dir in Get-ChildItem $pluginsDir -Directory) {
    $pkgJson = Join-Path $dir.FullName 'package.json'
    if (-not (Test-Path $pkgJson)) {
      # Empty dir = uninitialized submodule (fresh clone without
      # --recurse-submodules). That plugin will silently not load.
      if (-not (Get-ChildItem $dir.FullName -Force -ErrorAction SilentlyContinue | Select-Object -First 1)) {
        Write-StepErr "$($dir.Name): empty dir - uninitialized submodule? run: git submodule update --init"
        $failed += $dir.Name
      }
      continue
    }

    # Malformed package.json must not kill the whole setup run.
    try {
      $pkg = Get-Content $pkgJson -Raw | ConvertFrom-Json
    }
    catch {
      Write-StepErr "invalid package.json: $($dir.Name) ($($_.Exception.Message))"
      $failed += $dir.Name
      continue
    }

    $deps = $pkg.dependencies
    if (-not $deps -or -not ($deps.PSObject.Properties | Select-Object -First 1)) {
      Write-StepOk "$($dir.Name): no runtime deps"
      continue
    }

    # Frozen lockfile when committed (submodules), plain install otherwise
    # (caveman has no lockfile; --frozen-lockfile would fail there).
    $bunArgs = @('install', '--production')
    if ((Test-Path (Join-Path $dir.FullName 'bun.lock')) -or
        (Test-Path (Join-Path $dir.FullName 'bun.lockb'))) {
      $bunArgs += '--frozen-lockfile'
    }

    Write-Host "  bun $($bunArgs -join ' ') ($($dir.Name)) ..."
    Push-Location $dir.FullName
    # bun writes progress to stderr even on success; under the inherited
    # $ErrorActionPreference='Stop', stderr merged via 2>&1 becomes a
    # terminating error before $LASTEXITCODE is read (same guard as
    # Invoke-WingetInstall in common.ps1).
    $prevEAP = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $code = -1  # sentinel: stays -1 if the invocation itself throws
    try {
      & $PreFlight.Bun @bunArgs 2>&1 | ForEach-Object { Write-Host "    $_" }
      $code = $LASTEXITCODE
    }
    catch {
      Write-StepErr "bun invocation failed: $($dir.Name) ($($_.Exception.Message))"
    }
    finally {
      $ErrorActionPreference = $prevEAP
      Pop-Location
    }

    if ($code -ne 0) {
      Write-StepErr "bun install failed: $($dir.Name) (exit $code)"
      $failed += $dir.Name
      continue
    }
    Write-StepOk "$($dir.Name): deps installed"
  }

  if ($failed) { return 'failed' }
  return 'done'
}
