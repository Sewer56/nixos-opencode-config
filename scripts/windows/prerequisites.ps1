<#
.SYNOPSIS
  Pre-flight: detects cargo, bun, git, node, yarn, docker. Installs missing
  tools via winget (with per-tool consent) when run interactively.

.DESCRIPTION
  Returns an ordered hashtable of detected tools and their resolved paths.
  Keys: Cargo, Bun, Git, Node, Yarn, Docker (capitalised, as build modules
  access $PreFlight.Cargo / .Bun / .Git). Missing entries are $null.

  A missing tool triggers a y/N consent prompt (interactive shells only); on
  'yes' the tool is installed via winget, with a direct-installer fallback for
  cargo/bun/yarn, then re-detected. Docker Desktop needs elevation and is
  skipped with a clear message when the shell is not admin. Install failures
  are warned about, never fatal - callers skip affected phases and WARN
  instead. Only PowerShell 5.1+ is hard-required (#Requires enforces it).

  Non-interactive (piped) shells skip the consent prompt and behave like the
  old detect-and-warn flow so automated runs never block. Pass
  -NoInstallPrereqs on setup.ps1 to force detect-only even interactively.

  CodeRabbit CLI is intentionally NOT installed here: upstream ships only
  Linux/macOS binaries and WSL-only Windows support. The OpenCode agent
  config resolves `cr`/`coderabbit` at runtime and returns INCOMPLETE when
  absent (see config/agent/_review/coderabbit.md).

  Calls use the resolved binary paths returned here for determinism, rather
  than bare names that could be shadowed by Windows App Execution Aliases.

  Depends on helpers defined in common.ps1 (dot-sourced before this module
  by setup.ps1): Resolve-ToolPath, Get-Winget, Test-InteractiveConsole,
  Test-Admin, Confirm-Install, Invoke-WingetInstall, Update-CurrentProcessPath.
#>
#Requires -Version 5.1

# --- direct-installer fallbacks (used only when winget is unavailable/fails) --

# rustup: download rustup-init.exe from win.rustup.rs and run with -y.
# Still needs MSVC C++ build tools for `cargo build` to link (see MSVC probe).
# rustup-init installs the stable toolchain (cargo/rustc) to ~/.cargo/bin.
function Install-CargoDirect {
  $tmpDir = if ($env:TEMP) { $env:TEMP } else { [System.IO.Path]::GetTempPath() }
  $tmp = Join-Path $tmpDir 'rustup-init.exe'
  # Clear any half-downloaded/stale copy from a previous run before reusing.
  if (Test-Path $tmp) { Remove-Item $tmp -Force -ErrorAction SilentlyContinue }
  Write-Host '  fallback: downloading rustup-init.exe (win.rustup.rs) ...'
  # PS 5.1's progress bar makes Invoke-WebRequest ~10x slower; suppress it.
  $prevProgress = $ProgressPreference
  $ProgressPreference = 'SilentlyContinue'
  try {
    Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile $tmp -UseBasicParsing
  }
  catch {
    Write-StepWarn "rustup-init download failed: $($_.Exception.Message)"
    return $false
  }
  finally {
    $ProgressPreference = $prevProgress
  }
  # Run rustup-init under 'Continue' so its stderr lines (which it emits even
  # on success) are not promoted to terminating errors by the inherited Stop.
  $prevEAP = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    & $tmp -y 2>&1 | ForEach-Object { Write-Host "  $_" }
    $code = $LASTEXITCODE
  }
  finally {
    $ErrorActionPreference = $prevEAP
  }
  if ($code -eq 0) { return $true }
  Write-StepWarn "rustup-init.exe exited $code"
  return $false
}

# bun: official PowerShell installer (bun.sh/install.ps1).
function Install-BunDirect {
  Write-Host '  fallback: irm bun.sh/install.ps1 | iex ...'
  try {
    Invoke-Expression (Invoke-RestMethod 'https://bun.sh/install.ps1')
    return $true
  }
  catch {
    Write-StepWarn "bun install.ps1 failed: $($_.Exception.Message)"
    return $false
  }
}

# yarn (classic) via npm - only viable after node is available. Prefers the
# npm shim on PATH (installed with Node); no fragile npm-cli.js guessing.
function Install-YarnViaNpm {
  if (-not (Get-CommandPath 'node')) {
    Write-StepWarn 'cannot install yarn via npm: node not available yet'
    return $false
  }
  $npm = Get-CommandPath 'npm'
  if (-not $npm) {
    Write-StepWarn 'npm shim not found on PATH; cannot install yarn via npm'
    return $false
  }
  Write-Host '  fallback: npm install -g yarn ...'
  # npm routinely writes warnings to stderr; under inherited Stop those would
  # be promoted to terminating errors before $LASTEXITCODE is read.
  $prevEAP = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    & $npm install -g yarn 2>&1 | ForEach-Object { Write-Host "  $_" }
    $code = $LASTEXITCODE
  }
  finally {
    $ErrorActionPreference = $prevEAP
  }
  if ($code -eq 0) { return $true }
  Write-StepWarn "npm install -g yarn exited $code"
  return $false
}

# Dispatch a direct-installer fallback by tool command name. Returns $true on
# success. git/node/docker have no clean silent direct fallback - winget-only.
function Invoke-FallbackInstall([string]$Cmd) {
  switch ($Cmd) {
    'cargo' { return Install-CargoDirect }
    'bun'   { return Install-BunDirect }
    'yarn'  { return Install-YarnViaNpm }
    default {
      Write-StepWarn "no direct-installer fallback for '$Cmd' (winget-only)"
      return $false
    }
  }
}

# --- MSVC build tools probe (informational, never fatal) ---------------------
# Rust on Windows needs the MSVC linker (link.exe); rustup cannot provide it.
# Probe via vswhere for the VC.Tools.x86.x64 component. Absent => warn.
function Test-MSVCBuildTools {
  $pf86 = ${env:ProgramFiles(x86)}
  if (-not $pf86) { return $false }  # 32-bit Windows has no x86 Program Files
  $vswhere = Join-Path $pf86 'Microsoft Visual Studio\Installer\vswhere.exe'
  if (-not (Test-Path $vswhere)) { return $false }
  $out = & $vswhere -latest -products '*' `
    -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
    -property displayName 2>$null
  if ($LASTEXITCODE -eq 0 -and $out) { return $true }
  return $false
}

# --- per-tool detect + install loop ------------------------------------------
function Install-PrereqTool {
  param([hashtable]$Spec)

  $cmd  = $Spec.Cmd
  $path = Resolve-ToolPath $cmd $Spec.Candidates
  if ($path) { return $path }

  # Not found -> explain + offer install.
  Write-StepWarn "$cmd not found. $($Spec.Why)"
  Write-Host "         winget: $($Spec.WingetId)" -ForegroundColor DarkGray

  # Docker needs elevation; skip the prompt entirely when unprivileged so we
  # don't start a winget run that is guaranteed to fail. Use Contains() so a
  # missing NeedsAdmin key is explicit, not an implicit-null StrictMode trap.
  $needsAdmin = $Spec.Contains('NeedsAdmin') -and $Spec.NeedsAdmin
  if ($needsAdmin -and -not (Test-Admin)) {
    Write-StepWarn "$cmd needs an elevated shell for install; rerun setup.ps1 as admin."
    return $null
  }

  # Explicit escape hatch or non-interactive shell -> detect-only, no prompt.
  if ($script:NoInstallPrereqs) {
    Write-StepWarn "$cmd install skipped (-NoInstallPrereqs); affected phases will be skipped"
    return $null
  }
  if (-not (Test-InteractiveConsole)) {
    Write-StepWarn 'non-interactive shell; install skipped (detect-only mode)'
    return $null
  }
  # Consent. Read-Host lives inside Confirm-Install; guard it so a closed
  # console / Ctrl+C / unreadable stdin degrades to a WARN rather than
  # aborting the whole pre-flight under the inherited Stop preference.
  $confirmed = $false
  try { $confirmed = Confirm-Install $cmd $Spec.Why }
  catch { Write-StepWarn "consent prompt failed: $($_.Exception.Message)" }
  if (-not $confirmed) {
    Write-StepWarn "$cmd install declined or unavailable; affected phases will be skipped"
    return $null
  }

  # Install. winget may legitimately exit non-zero for "already installed",
  # so we gate the fallback on whether the tool is PRESENT after winget, not
  # on winget's exit code. The whole block is wrapped so any exception from
  # the native calls (web download, npm stderr under Stop, etc.) degrades to
  # a WARN instead of aborting the whole pre-flight (never-fatal invariant).
  try {
    $null = Invoke-WingetInstall $Spec.WingetId
    Update-CurrentProcessPath
    $path = Resolve-ToolPath $cmd $Spec.Candidates

    if (-not $path) {
      # winget didn't land the tool (absent, already-installed-via-stub, or
      # rustup installed without init'ing the toolchain). Try the direct
      # installer fallback - all are idempotent, so harmless if already done.
      Write-Host '  winget did not place the tool; trying direct-installer fallback ...' -ForegroundColor DarkGray
      $ok = Invoke-FallbackInstall $cmd
      if ($ok) {
        Update-CurrentProcessPath
        $path = Resolve-ToolPath $cmd $Spec.Candidates
      }
    }
  }
  catch {
    Write-StepWarn "$cmd install raised: $($_.Exception.Message)"
    $path = $null
  }

  if ($path) {
    Write-StepOk "$cmd installed: $path"
  }
  else {
    Write-StepErr "$cmd not available after install attempts; see messages above"
  }
  return $path
}

function Invoke-PreFlight {
  Write-PhaseHeader 'Pre-flight'

  # Tool specs: Key (hashtable key, capitalised for back-compat with build
  # modules), Cmd (exe name), WingetId, Why (human reason), Candidates (probe
  # paths when PATH refresh hasn't picked up the install), NeedsAdmin.
  $home_ = $env:USERPROFILE
  # Guard Join-Path: USERPROFILE is virtually always set on Windows, but
  # stripped CI/sandbox images can leave it empty -> Join-Path throws.
  $cargoCand = if ($home_) { @(Join-Path $home_ '.cargo\bin\cargo.exe') } else { @() }
  $bunCand   = if ($home_) { @(Join-Path $home_ '.bun\bin\bun.exe') }     else { @() }
  $specs = @(
    @{ Key='Cargo';  Cmd='cargo';  WingetId='Rustlang.Rustup';      Why='Rust toolchain (rustup) - builds the Rust workspace tools'; Candidates=$cargoCand },
    @{ Key='Bun';    Cmd='bun';    WingetId='Oven-sh.Bun';          Why='JavaScript runtime + bundler (OpenCode build)';            Candidates=$bunCand },
    @{ Key='Git';    Cmd='git';    WingetId='Git.Git';              Why='version control (OpenCode build cache key)';                Candidates=@('C:\Program Files\Git\cmd\git.exe') },
    @{ Key='Node';   Cmd='node';   WingetId='OpenJS.NodeJS.LTS';    Why='Node.js LTS (MCP server runtime)';                          Candidates=@('C:\Program Files\nodejs\node.exe') },
    @{ Key='Yarn';   Cmd='yarn';   WingetId='Yarn.Yarn';            Why='Yarn package manager (MCP runtime)';                         Candidates=@('C:\Program Files (x86)\Yarn\bin\yarn.cmd') },
    @{ Key='Docker'; Cmd='docker'; WingetId='Docker.DockerDesktop'; Why='Docker Desktop (container runtime)';                         Candidates=@('C:\Program Files\Docker\Docker\resources\bin\docker.exe'); NeedsAdmin=$true }
  )

  $results = [ordered]@{}
  foreach ($s in $specs) {
    $results[$s.Key] = (Install-PrereqTool $s)
  }

  # MSVC C++ build tools check (informational). Rust on Windows needs link.exe
  # from the "Desktop development with C++" VS workload; rustup cannot provide
  # it. Only meaningful when cargo is present. Never fatal.
  $results['MSVC'] = $null
  if ($results.Cargo) {
    if (Test-MSVCBuildTools) {
      Write-StepOk 'MSVC C++ build tools detected (Rust linker available)'
      $results['MSVC'] = 'present'
    }
    else {
      Write-StepWarn 'MSVC C++ build tools not detected.'
      Write-Host '         Rust on Windows needs link.exe from the "Desktop development' -ForegroundColor Yellow
      Write-Host '         with C++" VS workload (rustup cannot install it). Install via:' -ForegroundColor Yellow
      Write-Host '           winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"' -ForegroundColor DarkGray
      $results['MSVC'] = 'absent'
    }
  }

  # CodeRabbit CLI: not auto-installed (upstream is WSL-only on Windows).
  # The OpenCode /review/coderabbit agent returns INCOMPLETE when absent.
  $cr = Get-CommandPath 'cr'
  $coderabbit = Get-CommandPath 'coderabbit'
  if ($cr) {
    Write-StepOk "cr: $cr"
  }
  if ($coderabbit) {
    Write-StepOk "coderabbit: $coderabbit"
  }
  if (-not $cr -and -not $coderabbit) {
    Write-StepWarn 'CodeRabbit CLI not found - /review/coderabbit will return INCOMPLETE.'
    Write-Host '         Upstream ships Linux/macOS binaries only; on Windows install via WSL:' -ForegroundColor DarkGray
    Write-Host "           wsl -c 'curl -fsSL https://cli.coderabbit.ai/install.sh | sh'" -ForegroundColor DarkGray
  }

  # PowerShell version (informational; #Requires enforces 5.1)
  $psVer = $PSVersionTable.PSVersion
  Write-StepOk "PowerShell $psVer"

  return $results
}
