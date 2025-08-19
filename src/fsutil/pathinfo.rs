use std::{path::{Path, PathBuf}, time::SystemTime};
use super::util;

#[derive(Clone)]
pub struct PathInfo {
    pub path: PathBuf,
    pub file_name: String,     // lowercased
    pub ext: Option<String>,   // lowercased
    pub is_file: bool,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub age_secs: Option<u64>, // now - mtime
    pub hidden: bool,
}

impl PathInfo {
    pub fn from_entry(path: &Path, now: SystemTime) -> Option<Self> {
        let md = path.metadata().ok()?;
        let is_file = md.is_file();
        let is_dir  = md.is_dir();

        let file_name = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let ext = path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());

        let size = if is_file { Some(md.len()) } else { None };

        let age_secs = md.modified().ok()
            .and_then(|mt| now.duration_since(mt).ok())
            .map(|d| d.as_secs());

        let hidden = util::is_hidden(path);

        Some(Self { path: path.to_path_buf(), file_name, ext, is_file, is_dir, size, age_secs, hidden })
    }
    
    /// Build PathInfo from filesystem metadata.
    pub fn from_fs(path: &Path, now: SystemTime) -> Option<PathInfo> {
        let md = path.metadata().ok()?;
        let is_file = md.is_file();
        let is_dir = md.is_dir();

        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());

        let size = if is_file { Some(md.len()) } else { None };

        let age_secs = md
            .modified()
            .ok()
            .and_then(|mt| now.duration_since(mt).ok())
            .map(|d| d.as_secs());

        // Simple dotfile rule; replace if you have a platform-specific check.
        let hidden = file_name.starts_with('.');

        Some(PathInfo {
            path: path.to_path_buf(),
            file_name,
            ext,
            is_file,
            is_dir,
            size,
            age_secs,
            hidden,
        })
    }
}