use std::{path::PathBuf, time::SystemTime};
use walkdir::WalkDir;

use super::filter::Matcher;
use super::pathinfo::{PathInfo};
/// Iterator over filesystem entries applying a Matcher on normalized PathInfo.

pub struct FindIter<M: Matcher> {
    it: walkdir::IntoIter,
    matcher: M,
    now: SystemTime,
}

impl<M: Matcher> FindIter<M> {
    /// Create a new iterator rooted at `root` using the provided matcher.
    pub fn new(root: &str, matcher: M) -> Self {
        Self {
            it: WalkDir::new(root).into_iter(),
            matcher,
            now: SystemTime::now(),
        }
    }

    /// Create a new iterator with an injected `now` (useful for tests).
    pub fn with_now(root: &str, matcher: M, now: SystemTime) -> Self {
        Self {
            it: WalkDir::new(root).into_iter(),
            matcher,
            now,
        }
    }

    /// Access the internal matcher (read-only).
    pub fn matcher(&self) -> &M {
        &self.matcher
    }
}

impl<M: Matcher> Iterator for FindIter<M> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Ok(entry)) = self.it.next() {
            let path = entry.path();
            // Build PathInfo once and reuse; skip entries we cannot stat.
            if let Some(info) = PathInfo::from_fs(path, self.now) {
                if self.matcher.matches(&info) {
                    return Some(info.path);
                }
            }
        }
        None
    }
}