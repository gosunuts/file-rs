use clap::Args;
use std::io::{self, BufRead, Read};

#[derive(Args, Debug)]
pub struct MoveArgs {
    /// Source path (if omitted, will read from stdin)
    #[arg(short, long)]
    pub src: Option<String>,

    /// Destination path
    pub dst: String,

    /// Read NUL-delimited input from stdin (for find --print0 compatibility)
    #[arg(long)]
    pub stdin0: bool,
}

pub fn run(args: MoveArgs) {
    let sources: Vec<String> = if let Some(src) = args.src {
        vec![src]
    } else {
        if args.stdin0 {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf).expect("stdin read failed");
            buf.split(|b| *b == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .collect()
        } else {
            let stdin = io::stdin();
            stdin.lock().lines()
                .filter_map(Result::ok)
                .collect()
        }
    };

    for s in sources {
        println!("[mv] {} -> {}/{}", s, args.dst, file_name(&s));
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit_once('/').map(|(_, base)| base).unwrap_or(path)
}