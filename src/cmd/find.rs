use clap::Args;

#[derive(Args, Debug)]
pub struct FindArgs {
    /// Root directory to search
    #[arg(short, long, default_value = ".")]
    pub root: String,
}

pub fn run(args: FindArgs) {
    println!("[find] Root: {}", args.root);
}