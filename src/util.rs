
use std::path::{Path, PathBuf};
pub fn bundle_resource_url(plugin_name: &str, resource: &str) -> String {
    #[cfg(target_os = "windows")]
    { let c = windows_candidates(plugin_name, resource); for p in c { if p.exists() { return path_to_file_url(&p); } } return resource.to_string(); }
    #[cfg(target_os = "macos")]
    { let c = macos_candidates(plugin_name, resource); for p in c { if p.exists() { return path_to_file_url(&p); } } return resource.to_string(); }
}
#[cfg(target_os = "windows")]
fn windows_candidates(plugin_name: &str, resource: &str) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(program_files) = std::env::var("ProgramFiles") { v.push(Path::new(&program_files).join("Common Files").join("VST3").join(format!("{}.vst3", plugin_name)).join("Contents").join("Resources").join(resource)); }
    if let Ok(local) = std::env::var("LOCALAPPDATA") { v.push(Path::new(&local).join("Programs").join("Common").join("VST3").join(format!("{}.vst3", plugin_name)).join("Contents").join("Resources").join(resource)); }
    v
}
#[cfg(target_os = "macos")]
fn macos_candidates(plugin_name: &str, resource: &str) -> Vec<PathBuf> {
    let mut v = Vec::new();
    v.push(Path::new("/Library/Audio/Plug-Ins/VST3").join(format!("{}.vst3", plugin_name)).join("Contents").join("Resources").join(resource));
    if let Ok(home) = std::env::var("HOME") { v.push(Path::new(&home).join("Library/Audio/Plug-Ins/VST3").join(format!("{}.vst3", plugin_name)).join("Contents").join("Resources").join(resource)); }
    v
}
fn path_to_file_url(p: &Path) -> String { let s = p.to_string_lossy().replace('\\', "/"); format!("file:///{}", s) }
