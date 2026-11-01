$ErrorActionPreference = "Stop"

$binary = Join-Path $PSScriptRoot "magic-pointer.exe"
if (-not (Test-Path -LiteralPath $binary)) {
    $binary = Join-Path (Split-Path -Parent $PSScriptRoot) "magic-pointer.exe"
}

if (-not (Test-Path -LiteralPath $binary)) {
    throw "magic-pointer.exe is missing. Download the Windows x64 artifact from the GitHub Actions workflow and place it beside this script."
}

& $binary @args
exit $LASTEXITCODE
