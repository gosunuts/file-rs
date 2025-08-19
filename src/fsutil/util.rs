use std::path::Path;

pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// Parse human-friendly size like "100MB", "64KB", "1.5G".
/// Uses binary base (KiB=1024).
pub fn parse_human_size(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() { return None; }
    let (num, unit) = split_num_unit(s);
    let n: f64 = num.parse().ok()?;
    let mul = match unit.to_ascii_lowercase().as_str() {
        "" | "b"  => 1u64,
        "k" | "kb" => 1024u64,
        "m" | "mb" => 1024u64.pow(2),
        "g" | "gb" => 1024u64.pow(3),
        "t" | "tb" => 1024u64.pow(4),
        _ => return None,
    };
    Some((n * mul as f64) as u64)
}

/// Parse human-friendly age like "30s", "2h", "1d", "1.5w".
/// Returns seconds.
pub fn parse_human_age(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() { return None; }
    let (num, unit) = split_num_unit(s);
    let n: f64 = num.parse().ok()?;
    let secs = match unit.to_ascii_lowercase().as_str() {
        "" | "s" => 1.0,
        "m" => 60.0,
        "h" => 3600.0,
        "d" => 86400.0,
        "w" => 604800.0,
        _ => return None,
    };
    Some((n * secs) as u64)
}

/// Split "<number><unit>" into ("<number>", "<unit>").
fn split_num_unit(s: &str) -> (String, String) {
    let mut idx = 0;
    for (i, ch) in s.char_indices() {
        if !(ch.is_ascii_digit() || ch == '.') {
            idx = i;
            break;
        }
        idx = i + ch.len_utf8();
    }
    let (num, unit) = s.split_at(idx);
    (num.to_string(), unit.trim().to_string())
}