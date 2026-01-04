
param([string]$PluginName = "Vst3Skeleton")
$ErrorActionPreference = "Stop"
$bundleRoot = Join-Path (Get-Location).Path "$PluginName.vst3"
$dest = "C:\\Program Files\\Common Files\\VST3\\$PluginName.vst3"
Copy-Item $bundleRoot $dest -Recurse -Force
Write-Host "Installed to $dest"
