use super::pathinfo;

#[derive(Default, Clone)]
pub struct Filter {
    pub contains: Vec<String>,
    pub prefix:   Vec<String>,
    pub suffix:   Vec<String>,
    pub exts:     Vec<String>, // lowercased without dot
    pub ty_file:  bool,
    pub ty_dir:   bool,
    pub min_age_secs: Option<u64>,
    pub max_age_secs: Option<u64>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub include_hidden: bool,
}

pub trait Matcher {
    fn matches(&self, info: &pathinfo::PathInfo) -> bool;
}

impl Matcher for Filter {
    fn matches(&self, info: &pathinfo::PathInfo) -> bool {
        // type
        if self.ty_file && !info.is_file { return false; }
        if self.ty_dir  && !info.is_dir  { return false; }

        // hidden
        if !self.include_hidden && info.hidden { return false; }

        // name-based
        if !self.contains.is_empty()
            && !self.contains.iter().any(|s| info.file_name.contains(&s.to_ascii_lowercase())) {
            return false;
        }
        if !self.prefix.is_empty()
            && !self.prefix.iter().any(|p| info.file_name.starts_with(&p.to_ascii_lowercase())) {
            return false;
        }
        if !self.suffix.is_empty()
            && !self.suffix.iter().any(|s| info.file_name.ends_with(&s.to_ascii_lowercase())) {
            return false;
        }

        // ext
        if !self.exts.is_empty() {
            if let Some(ext) = &info.ext {
                if !self.exts.iter().any(|e| e == ext) { return false; }
            } else { return false; }
        }

        // size
        if let Some(min) = self.min_size {
            if info.size.map_or(true, |sz| sz < min) { return false; }
        }
        if let Some(max) = self.max_size {
            if info.size.map_or(true, |sz| sz > max) { return false; }
        }

        // age
        if let Some(min_age) = self.min_age_secs {
            if info.age_secs.map_or(true, |a| a < min_age) { return false; }
        }
        if let Some(max_age) = self.max_age_secs {
            if info.age_secs.map_or(true, |a| a > max_age) { return false; }
        }

        true
    }
}

pub fn merge(mut a: Filter, b: Filter) -> Filter {
    a.contains.extend(b.contains);
    a.prefix.extend(b.prefix);
    a.suffix.extend(b.suffix);
    a.exts.extend(b.exts);
    if b.ty_file || b.ty_dir { a.ty_file = b.ty_file; a.ty_dir = b.ty_dir; }
    if b.min_age_secs.is_some() { a.min_age_secs = b.min_age_secs; }
    if b.max_age_secs.is_some() { a.max_age_secs = b.max_age_secs; }
    if b.min_size.is_some()     { a.min_size = b.min_size; }
    if b.max_size.is_some()     { a.max_size = b.max_size; }
    if b.include_hidden         { a.include_hidden = true; }
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn mk_info(
        name: &str,
        ext: Option<&str>,
        is_file: bool,
        size: Option<u64>,
        age_secs: Option<u64>,
        hidden: bool,
    ) -> pathinfo::PathInfo {
        pathinfo::PathInfo {
            path: PathBuf::from(format!("/tmp/{}", name)),
            file_name: name.to_ascii_lowercase(),
            ext: ext.map(|e| e.to_ascii_lowercase()),
            is_file,
            is_dir: !is_file,
            size,
            age_secs,
            hidden,
        }
    }

    #[test]
    fn name_contains_prefix_suffix() {
        let mut f = Filter::default();
        f.contains = vec!["report".into()];
        f.prefix   = vec!["2024_".into()];
        f.suffix   = vec![".txt".into()];

        let ok = mk_info("2024_report.txt", Some("txt"), true, Some(10), Some(100), false);
        assert!(f.matches(&ok));

        let miss_contains = mk_info("2024_log.txt", Some("txt"), true, Some(10), Some(100), false);
        assert!(!f.matches(&miss_contains));

        let miss_prefix = mk_info("backup_report.txt", Some("txt"), true, Some(10), Some(100), false);
        assert!(!f.matches(&miss_prefix));

        let miss_suffix = mk_info("2024_report.md", Some("md"), true, Some(10), Some(100), false);
        assert!(!f.matches(&miss_suffix));
    }

    #[test]
    fn ext_filter() {
        let mut f = Filter::default();
        f.exts = vec!["txt".into(), "md".into()];

        let txt = mk_info("note.txt", Some("TXT"), true, Some(1), Some(1), false);
        let md  = mk_info("readme.md", Some("md"), true, Some(1), Some(1), false);
        let log = mk_info("app.log", Some("log"), true, Some(1), Some(1), false);
        let noext = mk_info("LICENSE", None, true, Some(1), Some(1), false);

        assert!(f.matches(&txt));
        assert!(f.matches(&md));
        assert!(!f.matches(&log));
        assert!(!f.matches(&noext));
    }

    #[test]
    fn type_filters() {
        let mut only_file = Filter::default();
        only_file.ty_file = true;

        let f1 = mk_info("a.txt", Some("txt"), true, Some(1), Some(1), false);
        let d1 = mk_info("dir", None, false, None, None, false);

        assert!(only_file.matches(&f1));
        assert!(!only_file.matches(&d1));

        // dir만 허용
        let mut only_dir = Filter::default();
        only_dir.ty_dir = true;

        assert!(!only_dir.matches(&f1));
        assert!(only_dir.matches(&d1));
    }

    #[test]
    fn hidden_behavior() {
        let f = Filter::default();
        let hidden_file = mk_info(".env", Some("env"), true, Some(1), Some(1), true);
        let visible_file = mk_info("env", Some("env"), true, Some(1), Some(1), false);

        assert!(!f.matches(&hidden_file));
        assert!(f.matches(&visible_file));

        let mut f2 = Filter::default();
        f2.include_hidden = true;
        assert!(f2.matches(&hidden_file));
    }

    #[test]
    fn size_bounds() {
        let mut f = Filter::default();
        f.min_size = Some(10);
        f.max_size = Some(100);

        let s5   = mk_info("a", Some("txt"), true, Some(5),   Some(1), false);
        let s10  = mk_info("b", Some("txt"), true, Some(10),  Some(1), false);
        let s50  = mk_info("c", Some("txt"), true, Some(50),  Some(1), false);
        let s100 = mk_info("d", Some("txt"), true, Some(100), Some(1), false);
        let s200 = mk_info("e", Some("txt"), true, Some(200), Some(1), false);

        assert!(!f.matches(&s5));
        assert!(f.matches(&s10));
        assert!(f.matches(&s50));
        assert!(f.matches(&s100));
        assert!(!f.matches(&s200));

        let dir = mk_info("dir", None, false, None, None, false);
        assert!(!f.matches(&dir));
    }

    #[test]
    fn age_bounds() {
        let mut f = Filter::default();
        f.min_age_secs = Some(60);
        f.max_age_secs = Some(3600);

        let a30   = mk_info("new",  Some("txt"), true, Some(1),  Some(30),   false);
        let a60   = mk_info("ok1",  Some("txt"), true, Some(1),  Some(60),   false);
        let a120  = mk_info("ok2",  Some("txt"), true, Some(1),  Some(120),  false);
        let a3600 = mk_info("ok3",  Some("txt"), true, Some(1),  Some(3600), false);
        let a7200 = mk_info("old",  Some("txt"), true, Some(1),  Some(7200), false);

        assert!(!f.matches(&a30));
        assert!(f.matches(&a60));
        assert!(f.matches(&a120));
        assert!(f.matches(&a3600));
        assert!(!f.matches(&a7200));

        let unknown_age = mk_info("x", Some("txt"), true, Some(1), None, false);
        assert!(!f.matches(&unknown_age));
    }

    #[test]
    fn merge_semantics() {
        let mut a = Filter::default();
        a.contains = vec!["log".into()];
        a.min_size = Some(10);

        let mut b = Filter::default();
        b.contains = vec!["error".into()];
        b.exts = vec!["txt".into()];
        b.max_size = Some(100);
        b.include_hidden = true;
        b.ty_file = true;

        let m = merge(a, b);
        assert_eq!(m.contains, vec!["log".to_string(), "error".to_string()]);
        assert_eq!(m.exts, vec!["txt".to_string()]);
        assert_eq!(m.min_size, Some(10));
        assert_eq!(m.max_size, Some(100));
        assert!(m.include_hidden);
        assert!(m.ty_file);
        assert!(!m.ty_dir);
    }

    #[test]
    fn combined_filters_pass_and_fail() {
        let mut f = Filter::default();
        f.ty_file = true;
        f.exts = vec!["txt".into()];
        f.contains = vec!["report".into()];
        f.min_size = Some(1);
        f.max_size = Some(100);
        f.min_age_secs = Some(10);
        f.max_age_secs = Some(1000);

        let ok = mk_info("monthly_report.txt", Some("txt"), true, Some(50), Some(200), false);
        assert!(f.matches(&ok));

        let bad_ext = mk_info("monthly_report.md", Some("md"), true, Some(50), Some(200), false);
        assert!(!f.matches(&bad_ext));

        let bad_name = mk_info("monthly_summary.txt", Some("txt"), true, Some(50), Some(200), false);
        assert!(!f.matches(&bad_name));

        let bad_size = mk_info("monthly_report.txt", Some("txt"), true, Some(1000), Some(200), false);
        assert!(!f.matches(&bad_size));

        let bad_age = mk_info("monthly_report.txt", Some("txt"), true, Some(50), Some(5), false);
        assert!(!f.matches(&bad_age));

        let hidden = mk_info(".monthly_report.txt", Some("txt"), true, Some(50), Some(200), true);
        assert!(!f.matches(&hidden));
    }
}