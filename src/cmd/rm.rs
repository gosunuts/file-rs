use clap::Args;
use std::fs;
use std::io::{self, BufRead, Read};
use std::path::{Path, PathBuf};
use tracing::{error, info};
use anyhow::{Result, anyhow};

#[derive(Args, Debug)]
pub struct RmArgs {
    /// Source path (if omitted, will read from stdin)
    #[arg(short, long)]
    pub src: Option<String>,

    #[arg(long)]
    pub stdin0: bool,

    /// Dry-run only (default true)
    #[arg(long, default_value_t = true)]
    pub dry_run: bool,   
}

pub fn run(args: RmArgs) {
    let sources: Vec<String> = if let Some(src) = args.src {
        vec![src]
    } else if args.stdin0 {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).expect("stdin read failed");
        buf.split(|b| *b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .collect()
    } else {
        let stdin = io::stdin();
        stdin.lock().lines().filter_map(Result::ok).collect()
    };

    let mut failures = 0usize;

    for s in sources {
        let p = PathBuf::from(&s);
        match rm_path(&p, args.dry_run) {
            Ok(false) => {
                info!(target: "file-rs", action="rm", dry_run=true, path=%p.display(), "Would remove");
            }
            Ok(true) => {
                info!(target: "file-rs", action="rm", dry_run=false, path=%p.display(), "Removed");
            }
            Err(e) => {
                failures += 1;
                error!(target: "file-rs", action="rm", path=%p.display(), error=%e, "Failed to remove");
            }
        }
    }

    if failures > 0 {
        std::process::exit(1);
    }
}

pub fn rm_path(path: &Path, dry_run: bool) -> io::Result<bool> {
    if dry_run {
        return Ok(false);
    }

    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => return Err(e),
    };

    if meta.is_file() {
        fs::remove_file(path)?;
        Ok(true)
    } else if meta.is_dir() {
        match fs::remove_dir(path) {
            Ok(()) => Ok(true),
            Err(e) => Err(e),
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "unsupported file type (symlink/special)",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn dry_run_does_not_delete_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, b"hello").unwrap();

        let res = rm_path(&file, true).unwrap();
        assert_eq!(res, false);
        assert!(file.exists());
    }

    #[test]
    fn apply_deletes_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, b"hello").unwrap();

        let res = rm_path(&file, false).unwrap();
        assert_eq!(res, true);
        assert!(!file.exists());
    }

    #[test]
    fn error_on_nonexistent_path() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("nope.txt");

        let err = rm_path(&missing, false).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn remove_empty_dir() {
        let dir = tempdir().unwrap();
        let empty = dir.path().join("empty");
        fs::create_dir(&empty).unwrap();

        let res = rm_path(&empty, false).unwrap();
        assert_eq!(res, true);
        assert!(!empty.exists());
    }

    #[test]
    fn non_empty_dir_fails() {
        let dir = tempdir().unwrap();
        let non_empty = dir.path().join("d");
        let inner = non_empty.join("x.txt");
        fs::create_dir(&non_empty).unwrap();
        fs::write(&inner, b"x").unwrap();

        let err = rm_path(&non_empty, false).unwrap_err();
        assert!(matches!(err.kind(), io::ErrorKind::Other | io::ErrorKind::PermissionDenied));
        assert!(non_empty.exists());
        assert!(inner.exists());
    }
}