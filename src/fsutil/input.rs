use std::io::{self, BufRead, Read};

pub fn iter_stdin_lines() -> impl Iterator<Item = String> {
    io::stdin().lock().lines().filter_map(Result::ok)
}

pub fn iter_stdin_nul() -> impl Iterator<Item = String> {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf).expect("stdin read failed");
    buf.split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect::<Vec<_>>()
        .into_iter()
}

pub fn iter_buf_nul<R: Read>(mut r: R) -> impl Iterator<Item = String> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).unwrap();
    buf.split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect::<Vec<_>>()
        .into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_iter_stdin_lines_like() {
        let input = b"foo.txt\nbar.txt\nbaz.log\n";
        let cursor = Cursor::new(input);

        let lines: Vec<String> = cursor
            .lines()
            .filter_map(Result::ok)
            .collect();

        assert_eq!(lines, vec!["foo.txt", "bar.txt", "baz.log"]);
    }

    #[test]
    fn test_iter_buf_nul() {
        let input = b"foo.txt\0bar.txt\0baz.log\0";
        let cursor = Cursor::new(input);

        let lines: Vec<String> = iter_buf_nul(cursor).collect();

        assert_eq!(lines, vec!["foo.txt", "bar.txt", "baz.log"]);
    }

    #[test]
    fn test_iter_buf_nul_with_spaces() {
        let input = b"foo bar.txt\0baz qux.log\0";
        let cursor = Cursor::new(input);

        let lines: Vec<String> = iter_buf_nul(cursor).collect();

        assert_eq!(lines, vec!["foo bar.txt", "baz qux.log"]);
    }
}
