# Generate Anchor IDL and copy to repo root `idl/` folder
Set-StrictMode -Version Latest
$here = Split-Path -Parent $MyInvocation.MyCommand.Definition
Set-Location "$here\..\programs\snowboard-depin"
Write-Host "Running anchor build..."
anchor build

$idl_src = Join-Path -Path "$here\..\programs\snowboard-depin\target\idl" -ChildPath "snowboard_depin.json"
$idl_dst_dir = Join-Path -Path "$here\.." -ChildPath "idl"
New-Item -ItemType Directory -Force -Path $idl_dst_dir | Out-Null
if (Test-Path $idl_src) {
    Copy-Item $idl_src -Destination (Join-Path $idl_dst_dir "snowboard_depin.json") -Force
    Write-Host "IDL copied to $idl_dst_dir\snowboard_depin.json"
} else {
    Write-Host "IDL not found at $idl_src. Ensure 'anchor' build succeeded." -ForegroundColor Yellow
}
