<#
.SYNOPSIS
  Shared helpers for the Windows setup scripts.

.DESCRIPTION
  Dot-sourced by setup.ps1; not run directly. All public helpers are
  idempotent. Repository root is in $script:RepoRoot (set by setup.ps1
  before this file is dot-sourced).
#>
#Requires -Version 5.1

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
