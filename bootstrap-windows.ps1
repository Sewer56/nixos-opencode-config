<#
.SYNOPSIS
  Windows bootstrap for the opencode config repo.

.DESCRIPTION
  Native Windows has no Nix / Home-Manager, so the two HM-managed symlinks
  created by flake.nix on Linux need a manual equivalent here:

    %USERPROFILE%\opencode           ->  repo root        ( == ~/opencode )
    %USERPROFILE%\.config\opencode   ->  repo \config\    ( == ~/.config/opencode )

  Directory junctions are used instead of symbolic links: junctions target
  directories only and require NO administrator rights and NO Developer Mode.
  OpenCode resolves its config dir via xdg-basedir, which falls back to
  %USERPROFILE%\.config on Windows when XDG_CONFIG_HOME is unset, so the
  second junction lands exactly where OpenCode looks.

  The script is idempotent: existing junctions are replaced; existing real
  directories are left untouched with a warning (never deletes user data).

.PARAMETER RepoRoot
  Repo root path. Defaults to this script's directory (script lives at repo root).

.PARAMETER UseEnvVar
  Skip the config junction and instead persist OPENCODE_CONFIG_DIR (user env)
  pointing at repo \config\. OpenCode honours this over the default dir.

.EXAMPLE
  pwsh ./bootstrap-windows.ps1
  pwsh ./bootstrap-windows.ps1 -UseEnvVar
#>
#Requires -Version 5.1
[CmdletBinding()]
param(
  [string]$RepoRoot = $PSScriptRoot,
  [switch]$UseEnvVar
)

$ErrorActionPreference = 'Stop'
$home_ = $env:USERPROFILE
if (-not $home_) { throw "USERPROFILE is not set; cannot locate user home." }

$ConfigTarget  = Join-Path $RepoRoot 'config'
$ShortcutLink  = Join-Path $home_ 'opencode'
$ConfigLink    = Join-Path $home_ '.config\opencode'

if (-not (Test-Path $RepoRoot -PathType Container)) {
  throw "RepoRoot does not exist: $RepoRoot"
}
if (-not (Test-Path $ConfigTarget -PathType Container)) {
  throw "config dir missing in repo: $ConfigTarget"
}

# --- helpers -----------------------------------------------------------------

function Test-IsJunction([string]$Path) {
  if (-not (Test-Path $Path)) { return $false }
  $item = Get-Item $Path -Force
  return ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) `
       -and ($item.LinkType -eq 'Junction')
}

# Remove a directory junction without recursing into its target contents.
# rmdir on a junction deletes only the reparse point.
function Remove-Junction([string]$Path) {
  cmd /c rmdir "$Path" | Out-Null
}

# Idempotently point $Link (junction) at $Target.
#   - missing      -> create junction
#   - junction     -> recreate (target may have moved)
#   - real dir     -> leave untouched, warn (never delete user data)
#   - file         -> error
function Set-Junction([string]$Link, [string]$Target, [string]$Label) {
  if (Test-IsJunction $Link) {
    Write-Host "replacing existing junction: $Label" -ForegroundColor Yellow
    Remove-Junction $Link
  }
  elseif (Test-Path $Link) {
    if (Test-Path $Link -PathType Container) {
      Write-Warning "$Label already exists as a real directory; leaving untouched: $Link"
      return
    }
    throw "$Label target is a file, not a junction/dir; refusing to overwrite: $Link"
  }

  $parent = Split-Path -Parent $Link
  if ($parent -and -not (Test-Path $parent)) {
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
  }

  New-Item -ItemType Junction -Path $Link -Target $Target | Out-Null
  Write-Host "junction: $Label" -ForegroundColor Green
  Write-Host "   $Link" -ForegroundColor DarkGray
  Write-Host "   -> $Target" -ForegroundColor DarkGray
}

# --- links -------------------------------------------------------------------

Set-Junction $ShortcutLink $RepoRoot     'repo shortcut (~\opencode)'

if ($UseEnvVar) {
  # Alternative to the config junction: tell OpenCode where the config lives.
  [Environment]::SetEnvironmentVariable('OPENCODE_CONFIG_DIR', $ConfigTarget, 'User')
  $env:OPENCODE_CONFIG_DIR = $ConfigTarget
  Write-Host "set OPENCODE_CONFIG_DIR (User) = $ConfigTarget" -ForegroundColor Green
  Write-Host 'Open a new shell for the env var to take effect.' -ForegroundColor Yellow
}
else {
  Set-Junction $ConfigLink $ConfigTarget 'opencode config (~\.config\opencode)'
}

# --- secrets check -----------------------------------------------------------
# opencode.json references these via {file:~/.secrets/<name>}. Missing file =>
# OpenCode InvalidError at load. Create the dir; warn about absent files.

$SecretsDir = Join-Path $home_ '.secrets'
$Expected = 'axonhub-url', 'axonhub-key', 'axonhub-work-key', 'github-token'

if (-not (Test-Path $SecretsDir)) {
  New-Item -ItemType Directory -Path $SecretsDir | Out-Null
  Write-Host "created secrets dir: $SecretsDir" -ForegroundColor Green
}

$missing = $Expected | Where-Object { -not (Test-Path (Join-Path $SecretsDir $_)) }
if ($missing) {
  Write-Warning ("Missing secret file(s) in {0}: {1}" -f $SecretsDir, ($missing -join ', '))
  Write-Host 'OpenCode will refuse to load until these exist.' -ForegroundColor Yellow
}
else {
  Write-Host 'all expected secret files present.' -ForegroundColor Green
}

Write-Host "`nDone." -ForegroundColor Green
