param (
    [switch]$Release,
    [string]$Dest
)

$ErrorActionPreference = "Stop"

$configuration = "debug"
if ($Release) {
    $configuration = "release"
}

Write-Host "Building Rust project ($configuration)..."
if ($Release) {
    cargo build --release
} else {
    cargo build
}

if (${LASTEXITCODE} -ne 0) {
    Write-Error "Build failed with exit code ${LASTEXITCODE}"
}

$destDir = "$env:LOCALAPPDATA\Programs\EmEditor\PlugIns"
if ($Dest) {
    $destDir = $Dest
}

if (!(Test-Path $destDir)) {
    Write-Host "Creating Plugins directory: $destDir"
    New-Item -ItemType Directory -Path $destDir | Out-Null
}

$src = "target\$configuration\emeditor_terminal.dll"
$destFile = "$destDir\emeditor_terminal.dll"

if (!(Test-Path $src)) {
    Write-Error "Build artifact not found: $src"
}

Write-Host "Copying plugin to $destDir..."
Copy-Item -Path $src -Destination $destFile -Force

Write-Host "Done. Please restart EmEditor to load the plugin."