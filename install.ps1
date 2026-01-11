$ErrorActionPreference = "Stop"

$configuration = "debug"
if ($args.Contains("--release")) {
    $configuration = "release"
}

Write-Host "Building Rust project ($configuration)..."
if ($configuration -eq "release") {
    cargo build --release
} else {
    cargo build
}

$destDir = "$env:APPDATA\Emurasoft\EmEditor\PlugIns"
if (!(Test-Path $destDir)) {
    Write-Host "Creating Plugins directory: $destDir"
    New-Item -ItemType Directory -Path $destDir | Out-Null
}

$src = "target\$configuration\emeditor_terminal.dll"
$dest = "$destDir\emeditor_terminal.dll"

if (!(Test-Path $src)) {
    Write-Error "Build artifact not found: $src"
}

Write-Host "Copying plugin to $destDir..."
Copy-Item -Path $src -Destination $dest -Force

Write-Host "Done. Please restart EmEditor to load the plugin."
