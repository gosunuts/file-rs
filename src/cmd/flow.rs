use clap::Args;
use std::str::FromStr;

use super::{find, mv, compress, rm};


#[derive(Args, Debug)]
pub struct FlowArgs {
    /// Find phase options
    #[command(flatten)]
    pub find: find::FindArgs,

    /// Actions to apply in order, e.g.:
    ///   mv:to=dst
    ///   compress:dst=archive.tar.zst
    ///   copy:to=backup/
    ///   rm:trash=true
    ///
    /// --action 'mv:to=dst' --action 'rm:trash=true'
    #[arg(long = "action")]
    pub actions: Vec<ActionSpec>,

    /// Dry-run
    #[arg(long, default_value_t = true)]
    pub dry_run: bool,
}
#[derive(Debug, Clone)]
pub enum ActionSpec {
    Mv { to: String },
    Copy { to: String },
    Compress { dst: String },
    Rm { trash: bool },
}

impl FromStr for ActionSpec {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // kind:key=value[,key=value...]
        // "mv:to=dst", "rm:trash=true", "compress:dst=out.tar.zst"
        let (kind, rest) = s.split_once(':').ok_or("expected kind:kv syntax")?;
        let mut to: Option<String> = None;
        let mut dst: Option<String> = None;
        let mut trash: Option<bool> = None;

        for kv in rest.split(',') {
            let (k, v) = kv.split_once('=').ok_or("expected key=value")?;
            match (kind, k) {
                ("mv", "to") => to = Some(v.to_string()),
                ("copy", "to") => to = Some(v.to_string()),
                ("compress", "dst") => dst = Some(v.to_string()),
                ("rm", "trash") => {
                    trash = Some(match v {
                        "1" | "true" | "yes" => true,
                        "0" | "false" | "no" => false,
                        _ => return Err("trash expects true/false".into()),
                    })
                }
                _ => return Err(format!("unsupported key '{}' for kind '{}'", k, kind)),
            }
        }

        match kind {
            "mv" => Ok(ActionSpec::Mv { to: to.ok_or("mv requires to=<path>")? }),
            "copy" => Ok(ActionSpec::Copy { to: to.ok_or("copy requires to=<path>")? }),
            "compress" => Ok(ActionSpec::Compress { dst: dst.ok_or("compress requires dst=<file>")? }),
            "rm" => Ok(ActionSpec::Rm { trash: trash.unwrap_or(true) }),
            _ => Err(format!("unknown action kind: {}", kind)),
        }
    }
}

pub fn run(args: FlowArgs) {
    println!("[flow] find root={} (dry-run={})", args.find.root, args.dry_run);

    let demo_files = vec![
        "a.jpg".to_string(),
        "b.jpg".to_string(),
        "nested/c.jpg".to_string(),
    ];

    for f in demo_files {
        let mut current = f.clone();
        for act in &args.actions {
            match act {
                ActionSpec::Mv { to } => {
                    let dst = format!("{}/{}", to, file_name(&current));
                    if args.dry_run {
                        println!("[flow] [dry-run] mv {} -> {}", current, dst);
                    } else {
                        println!("[flow] mv {} -> {}", current, dst);
                    }
                    current = dst;
                }
                ActionSpec::Copy { to } => {
                    let dst = format!("{}/{}", to, file_name(&current));
                    if args.dry_run {
                        println!("[flow] [dry-run] cp {} -> {}", current, dst);
                    } else {
                        println!("[flow] cp {} -> {}", current, dst);
                    }
                }
                ActionSpec::Compress { dst } => {
                    if args.dry_run {
                        println!("[flow] [dry-run] compress add {} -> {}", current, dst);
                    } else {
                        println!("[flow] compress add {} -> {}", current, dst);
                    }
                }
                ActionSpec::Rm { trash } => {
                    if args.dry_run {
                        println!("[flow] [dry-run] rm {} (trash={})", current, trash);
                    } else {
                        println!("[flow] rm {} (trash={})", current, trash);
                    }
                }
            }
        }
    }

    println!("[flow] done.");
}

fn file_name(p: &str) -> &str {
    p.rsplit_once('/').map(|(_, base)| base).unwrap_or(p)
}