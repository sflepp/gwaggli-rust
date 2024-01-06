use dirs::{cache_dir, home_dir};
use std::path::PathBuf;

#[allow(dead_code)]
pub fn gwaggli_home_dir() -> PathBuf {
    home_dir().unwrap().join("gwaggli-rs")
}
pub fn gwaggli_cache_dir() -> PathBuf {
    cache_dir().unwrap().join("gwaggli-rs")
}
pub fn models_dir() -> PathBuf {
    gwaggli_cache_dir().join("models")
}
pub fn download_cache_dir() -> PathBuf {
    gwaggli_cache_dir().join("download")
}
pub fn clear_cache() {
    let _ = std::fs::remove_dir_all(gwaggli_cache_dir());
}
pub fn prepare_download_cache_dir() {
    let _ = std::fs::create_dir_all(download_cache_dir());
}
