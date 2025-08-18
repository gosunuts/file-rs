use clap::Args;

#[derive(Args, Debug)]
pub struct RmArgs {
    /// Path to remove
    pub path: String,

    /// Dry-run only (default true)
    #[arg(long, default_value_t = true)]
    pub dry_run: bool,
}

pub fn run(args: RmArgs) {
    if args.dry_run {
        println!("[rm] Dry-run: would remove {}", args.path);
    } else {
        println!("[rm] Removing {}", args.path);
    }
}