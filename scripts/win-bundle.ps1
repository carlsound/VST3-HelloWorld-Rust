
param(
  [string]$CrateDir = "vst3_skeleton_db_vertical_meters_tooltips",
  [string]$PluginName = "Vst3Skeleton"
)
$ErrorActionPreference = "Stop"
Push-Location $CrateDir
cargo build --release
Pop-Location
$root = (Get-Location).Path
$target = Join-Path $root "$CrateDir\target\release"
$dll = Join-Path $target "vst3_skeleton_db_vertical_meters_tooltips.dll"
$bundleRoot = Join-Path $root "$PluginName.vst3"
$contents = Join-Path $bundleRoot "Contents"
$arch = Join-Path $contents "x86_64-win"
$resources = Join-Path $contents "Resources"
New-Item -ItemType Directory -Force -Path $arch | Out-Null
New-Item -ItemType Directory -Force -Path $resources | Out-Null
$dstDll = Join-Path $arch "$PluginName.vst3"
Copy-Item $dll $dstDll -Force
Copy-Item (Join-Path $root "$CrateDir\Resources\*") $resources -Recurse -Force
Write-Host "Bundled at $bundleRoot"
