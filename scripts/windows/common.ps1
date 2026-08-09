<#
.SYNOPSIS
  Shared helpers for the Windows setup scripts.

.DESCRIPTION
  Dot-sourced by setup.ps1; not run directly. All public helpers are
  idempotent. Repository root is in $script:RepoRoot (set by setup.ps1
  before this file is dot-sourced).
#>
#Requires -Version 5.1

# Force TLS 1.2 for the direct-installer web fallbacks (rustup-init, bun
# install.ps1). Windows PowerShell 5.1 / .NET Framework defaults to TLS 1.0,
# which Cloudflare-fronted hosts (bun.sh, win.rustup.rs) refuse. PS 7
# ignores this harmlessly.
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# --- logging -----------------------------------------------------------------

function Write-PhaseHeader([string]$Name) {
  Write-Host "`n=== $Name ===" -ForegroundColor Cyan
}

function Write-StepOk([string]$Message) {
  Write-Host "  [OK]   $Message" -ForegroundColor Green
}

function Write-StepWarn([string]$Message) {
  Write-Host "  [WARN] $Message" -ForegroundColor Yellow
}

function Write-StepErr([string]$Message) {
  Write-Host "  [ERR]  $Message" -ForegroundColor Red
}

# --- command detection -------------------------------------------------------

function Get-CommandPath([string]$Name) {
  $cmd = Get-Command $Name -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Source }
  return $null
}

# Resolve a tool by name on PATH, else by probing candidate absolute paths.
# Used after an install when Get-Command may not yet see the new PATH entry
# (installers register PATH in the User/Machine scope, not the live process).
# Skips Windows App Execution Alias stubs under %LOCALAPPDATA%\Microsoft\
# WindowsApps - those are Store redirectors, not real binaries, and would
# otherwise shadow a genuine install for yarn/node/etc.
function Resolve-ToolPath([string]$Name, [string[]]$Candidates) {
  $p = Get-CommandPath $Name
  if ($p -and ($p -notmatch 'Microsoft\\WindowsApps\\')) { return $p }
  if ($Candidates) {
    foreach ($c in $Candidates) {
      if ($c -and (Test-Path $c)) { return $c }
    }
  }
  return $null
}

# --- prerequisite installation -----------------------------------------------
# Auto-installs missing tools via winget (with per-tool consent) when the
# pre-flight runs interactively. Non-interactive/piped shells fall back to
# the old detect-and-warn behaviour so scripted runs never block on a prompt.
# Direct-installer fallbacks run only when winget is unavailable or fails.

# Locate winget.exe (App Installer). $null if absent (older Win10 / App
# Installer not present). Never throws.
function Get-Winget {
  $w = Get-Command winget -ErrorAction SilentlyContinue
  if ($w) { return $w.Source }
  return $null
}

# True when stdin AND stdout are NOT redirected (i.e. a real interactive
# console the user can both read prompts on and type into). Gates consent
# prompts: piped/non-interactive runs skip installs so they never block.
function Test-InteractiveConsole {
  try {
    return -not ([Console]::IsInputRedirected -or [Console]::IsOutputRedirected)
  }
  catch { return $false }  # headless/unknown -> treat as non-interactive
}

# True when the current process is elevated (admin). Needed because Docker
# Desktop is a machine-scope install that winget cannot complete unprivileged.
function Test-Admin {
  try {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    $p  = New-Object Security.Principal.WindowsPrincipal($id)
    return $p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
  } catch { return $false }
}

# y/N prompt, default No. Returns $true only on an explicit yes.
function Confirm-Install([string]$Tool, [string]$Reason) {
  Write-Host "  Missing: $Tool ($Reason)" -ForegroundColor Yellow
  $resp = Read-Host "  Install now? [y/N]"
  return ($resp -match '^[yY]')
}

# Run `winget install` for a package id. Accepts source/package agreements
# and uses silent mode to avoid installer GUI popups. Returns $true only when
# winget exited 0 (caller re-probes the tool regardless, since winget returns
# non-zero for "already installed" which is a success state). Idempotent.
function Invoke-WingetInstall([string]$PackageId) {
  $winget = Get-Winget
  if (-not $winget) {
    Write-StepWarn 'winget not found (App Installer missing); cannot winget install'
    return $false
  }
  # winget prints progress on stderr; merged with 2>&1 under
  # $ErrorActionPreference='Stop' (inherited from setup.ps1) those can be
  # promoted to terminating errors before $LASTEXITCODE is read. Run this
  # block with 'Continue'; restore on exit (function-scoped, no caller bleed).
  $prevEAP = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    $wingetArgs = @(
      'install','--id',$PackageId,'-e','--silent',
      '--accept-source-agreements','--accept-package-agreements'
    )
    # Docker Desktop's bootstrapper ignores winget --silent; pass its own
    # --quiet via --override so it doesn't pop a GUI/UAC prompt.
    if ($PackageId -eq 'Docker.DockerDesktop') {
      $wingetArgs += '--override','--quiet'
    }
    Write-Host "  winget install --id $PackageId -e --silent ..."
    $out = & $winget @wingetArgs 2>&1
    $code = $LASTEXITCODE
    $out | ForEach-Object { Write-Host "  $_" }
  }
  finally {
    $ErrorActionPreference = $prevEAP
  }
  if ($code -eq 0) {
    Write-StepOk "winget install exited 0: $PackageId"
    return $true
  }
  # Non-zero may be "already installed" (success) or a real failure; the
  # caller re-probes the tool and only falls back if still absent, so a
  # misleading "failed" here never causes an incorrect skip. Wrap $code in
  # $() so the ':' that follows is not parsed as a scope separator.
  Write-StepWarn "winget install exit $($code): $PackageId (re-probing tool)"
  return $false
}

# Rebuild $env:Path from persisted Machine + User PATH (and preserve any
# process-only entries) so tools installed earlier in this same run become
# visible without opening a new shell. Dedupes to avoid PATH bloat on re-run.
function Update-CurrentProcessPath {
  $machine = [Environment]::GetEnvironmentVariable('Path','Machine')
  $user    = [Environment]::GetEnvironmentVariable('Path','User')
  $parts = @()
  if ($machine) { $parts += ($machine -split ';' | Where-Object { $_ -ne '' }) }
  if ($user)    { $parts += ($user    -split ';' | Where-Object { $_ -ne '' }) }
  if ($env:Path){ $parts += ($env:Path -split ';' | Where-Object { $_ -ne '' }) }
  $env:Path = (($parts | Select-Object -Unique) -join ';')
}

# --- junctions ----------------------------------------------------------------
# Directory junctions target directories only, require NO admin rights and NO
# Developer Mode. Idempotent: existing junctions replaced; existing real
# directories are left untouched with a warning (never deletes user data).

function Test-IsJunction([string]$Path) {
  if (-not (Test-Path $Path)) { return $false }
  $item = Get-Item $Path -Force
  return ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) `
       -and ($item.LinkType -eq 'Junction')
}

# Remove a junction without recursing into its target. rmdir on a junction
# deletes only the reparse point. Surface rmdir failures (locked path, ACL
# issue) instead of letting Set-Junction later fail with a confusing
# "already exists" error that hides the original cause.
function Remove-Junction([string]$Path) {
  cmd /c rmdir "$Path" | Out-Null
  if ($LASTEXITCODE -ne 0) {
    throw "rmdir failed to remove junction: $Path (exit $LASTEXITCODE)"
  }
}

function Set-Junction([string]$Link, [string]$Target, [string]$Label) {
  if (Test-IsJunction $Link) {
    Write-Host "  replacing existing junction: $Label" -ForegroundColor Yellow
    Remove-Junction $Link
  }
  elseif (Test-Path $Link) {
    if (Test-Path $Link -PathType Container) {
      Write-StepWarn "$Label already exists as a real directory; leaving untouched: $Link"
      return
    }
    throw "$Label target is a file, not a junction/dir; refusing to overwrite: $Link"
  }

  $parent = Split-Path -Parent $Link
  if ($parent -and -not (Test-Path $parent)) {
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
  }

  New-Item -ItemType Junction -Path $Link -Target $Target | Out-Null
  Write-Host "  junction: $Label" -ForegroundColor Green
  Write-Host "     $Link" -ForegroundColor DarkGray
  Write-Host "  -> $Target" -ForegroundColor DarkGray
}

# --- user environment vars ---------------------------------------------------

function Set-UserEnvVar([string]$Name, [string]$Value) {
  $current = [Environment]::GetEnvironmentVariable($Name, 'User')
  if ($current -eq $Value) {
    Write-StepOk "$Name already set (User scope)"
    return
  }
  [Environment]::SetEnvironmentVariable($Name, $Value, 'User')
  # Mirror into the current process so subsequent phases see it too.
  Set-Item -Path "Env:$Name" -Value $Value
  Write-StepOk "set $Name (User scope) = $Value"
}

function Add-ToUserPath([string]$Dir) {
  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  if (-not $userPath) { $userPath = '' }
  $parts = $userPath -split ';' | Where-Object { $_ -ne '' }
  if ($parts -contains $Dir) {
    Write-StepOk "already on User PATH: $Dir"
    return
  }
  $newUserPath = if ($userPath) { $userPath.TrimEnd(';') + ';' + $Dir } else { $Dir }
  [Environment]::SetEnvironmentVariable('Path', $newUserPath, 'User')
  # Mirror into the current process so later phases in this same run see it
  # (parity with Set-UserEnvVar, which mirrors via Set-Item Env:).
  $procParts = $env:Path -split ';' | Where-Object { $_ -ne '' }
  if ($procParts -notcontains $Dir) {
    $env:Path = if ($env:Path) { $env:Path.TrimEnd(';') + ';' + $Dir } else { $Dir }
  }
  Write-StepOk "added to User PATH: $Dir"
  Write-Host '  Open a new shell for the PATH change to take effect in other terminals.' -ForegroundColor Yellow
}

# --- file ops ----------------------------------------------------------------

# Copy $Source to $Destination, skipping when $Destination is at least as new
# as $Source (unless -Force). Returns $true on success, $false if source missing.
function Copy-FileIfNewer([string]$Source, [string]$Destination, [switch]$Force) {
  if (-not (Test-Path $Source)) {
    Write-StepErr "source missing: $Source"
    return $false
  }
  $leaf = Split-Path $Destination -Leaf
  if ((-not $Force) -and (Test-Path $Destination)) {
    $srcTime = (Get-Item $Source).LastWriteTime
    $dstTime = (Get-Item $Destination).LastWriteTime
    if ($srcTime -le $dstTime) {
      Write-StepOk "up to date, skipping: $leaf"
      return $true
    }
  }
  Copy-Item $Source $Destination -Force
  Write-StepOk "copied: $leaf"
  return $true
}
