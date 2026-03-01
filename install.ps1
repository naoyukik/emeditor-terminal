param (
    [Parameter(Position=0)]
    [Alias("Dest", "path")]
    [string]$OutputPath,

    [switch]$Release
)

$ErrorActionPreference = "Stop"

$configuration = "debug"
if ($Release) {
    $configuration = "release"
}

# ユーザーが誤って '--path' という文字列自体を値として渡してしまった場合のガード
if ($OutputPath -eq "--path" -or $OutputPath -eq "-path") {
    Write-Error "Invalid path: '$OutputPath'. Please use '-Path ""C:\path\to\dll""' or simply '""C:\path\to\dll""'."
}

Write-Host "Building Rust project ($configuration)..."
if ($Release) {
    cargo build --release
} else {
    cargo build
}

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed with exit code ${LASTEXITCODE}"
}

# デフォルトの出力先
$destDir = "$env:LOCALAPPDATA\Programs\EmEditor\PlugIns"
$destFile = Join-Path $destDir "emeditor_terminal.dll"

if ($OutputPath) {
    # 指定されたパスを判定
    if (Test-Path $OutputPath -PathType Container) {
        # 既存のディレクトリが指定された場合
        $destDir = $OutputPath
        $destFile = Join-Path $destDir "emeditor_terminal.dll"
    } elseif ($OutputPath.ToLower().EndsWith(".dll")) {
        # .dll ファイルパスが指定された場合
        $destFile = $OutputPath
        $destDir = Split-Path -Parent $OutputPath
        if (-not $destDir) { $destDir = "." }
    } else {
        # それ以外はディレクトリ指定とみなす
        $destDir = $OutputPath
        $destFile = Join-Path $destDir "emeditor_terminal.dll"
    }
}

# パスを正規化（絶対パスへ）
try {
    $destDir = [System.IO.Path]::GetFullPath($destDir)
    $destFile = [System.IO.Path]::GetFullPath($destFile)
} catch {
    Write-Error "Failed to resolve path: $OutputPath"
}

Write-Host "Target directory: $destDir"
Write-Host "Target file: $destFile"

if (!(Test-Path $destDir)) {
    Write-Host "Creating directory: $destDir"
    New-Item -ItemType Directory -Path $destDir -Force | Out-Null
}

$src = "target\$configuration\emeditor_terminal.dll"

if (!(Test-Path $src)) {
    Write-Error "Build artifact not found: $src"
}

Write-Host "Copying plugin to $destFile..."
Copy-Item -Path $src -Destination $destFile -Force

Write-Host "Done. Please restart EmEditor to load the plugin."
