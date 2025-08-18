use std::fs;
use std::path::{Path, PathBuf};

pub fn file_name(path: &str) -> &str {
    path.rsplit_once('/').map(|(_, base)| base).unwrap_or(path)
}

pub fn normalize(path: &str) -> PathBuf {
    Path::new(path).to_path_buf()
}

pub fn move_one(src: &str, dst: &str) -> std::io::Result<()> {
    fs::rename(Path::new(src), Path::new(dst))
}

pub fn remove_one(path: &str) -> std::io::Result<()> {
    fs::remove_file(Path::new(path))
}