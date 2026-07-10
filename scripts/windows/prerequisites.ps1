<#
.SYNOPSIS
  Pre-flight: detects cargo, bun, git. Warns on missing, never fatal.

.DESCRIPTION
  Returns an ordered hashtable of detected tools and their resolved paths.
  Missing tools are NOT fatal - callers skip affected phases and WARN instead.
  Only PowerShell 5.1+ is hard-required (the #Requires directive enforces it).

  Calls use the resolved binary paths returned here for determinism, rather
  than bare names that could be shadowed by Windows App Execution Aliases.
#>
#Requires -Version 5.1

function Invoke-PreFlight {
  Write-PhaseHeader 'Pre-flight'

  $results = [ordered]@{
    Cargo = $null
    Bun   = $null
    Git   = $null
  }

  # cargo
  $cargo = Get-CommandPath 'cargo'
  if ($cargo) {
    $results.Cargo = $cargo
    Write-StepOk "cargo: $cargo"
  }
  else {
    Write-StepWarn 'cargo not found - Rust tools build will be skipped.'
    Write-Host '         Install via https://rustup.rs/' -ForegroundColor DarkGray
  }

  # bun
  $bun = Get-CommandPath 'bun'
  if ($bun) {
    $results.Bun = $bun
    Write-StepOk "bun: $bun"
  }
  else {
    Write-StepWarn 'bun not found - OpenCode build will be skipped.'
    Write-Host '         Install via https://bun.sh/ (or: npm i -g bun)' -ForegroundColor DarkGray
  }

  # git
  $git = Get-CommandPath 'git'
  if ($git) {
    $results.Git = $git
    Write-StepOk "git: $git"
  }
  else {
    Write-StepWarn 'git not found - OpenCode build cache disabled (always rebuild).'
  }

  # PowerShell version (informational; #Requires enforces 5.1)
  $psVer = $PSVersionTable.PSVersion
  Write-StepOk "PowerShell $psVer"

  return $results
}
