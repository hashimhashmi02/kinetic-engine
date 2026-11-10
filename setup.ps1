# Kinetic dev setup for Windows.
# Usage: powershell -ExecutionPolicy Bypass -File setup.ps1
# Checks required tools and installs Claude Code plugins at project scope.

$ErrorActionPreference = 'Continue'
$missing = @()

function Test-Tool($name, $hint) {
    $cmd = Get-Command $name -ErrorAction SilentlyContinue
    if ($cmd) {
        Write-Host ("  ok       {0}" -f $name)
    } else {
        Write-Host ("  MISSING  {0}  ->  {1}" -f $name, $hint)
        $script:missing += $name
    }
}

Write-Host 'Tools'
Test-Tool 'git'    'https://git-scm.com'
Test-Tool 'cargo'  'https://rustup.rs'
Test-Tool 'rustup' 'https://rustup.rs'
Test-Tool 'ffmpeg' 'scoop install ffmpeg   (needed for MP4 export)'
Test-Tool 'node'   'https://nodejs.org      (needed for hooks and the site)'
Test-Tool 'pnpm'   'npm install -g pnpm     (needed for the site)'
Test-Tool 'gh'     'https://cli.github.com  (needed for CI checks)'

if (Get-Command rustup -ErrorAction SilentlyContinue) {
    Write-Host 'Rust components'
    rustup component add rustfmt clippy rust-analyzer | Out-Null
    Write-Host '  ok       rustfmt, clippy, rust-analyzer'
}

Write-Host 'Claude Code plugins (project scope)'
if (Get-Command claude -ErrorAction SilentlyContinue) {
    $plugins = @(
        'rust-analyzer-lsp',
        'typescript-lsp',
        'code-review',
        'feature-dev',
        'commit-commands',
        'frontend-design'
    )
    foreach ($p in $plugins) {
        claude plugin install "$p@claude-plugins-official" --scope project *> $null
        if ($LASTEXITCODE -eq 0) {
            Write-Host ("  ok       {0}" -f $p)
        } else {
            Write-Host ("  FAILED   {0}  ->  run /plugin inside claude and install it by hand" -f $p)
        }
    }
} else {
    Write-Host '  MISSING  claude  ->  https://claude.com/claude-code'
    $missing += 'claude'
}

if ($missing.Count -gt 0) {
    Write-Host ("`nMissing: {0}" -f ($missing -join ', '))
    exit 1
}
Write-Host "`nReady. Start with: claude, then /milestone 1"
