use clap::Args;

#[derive(Args, Debug)]
pub struct CompressArgs {
    /// Source path to compress
    pub src: String,

    /// Destination archive path
    pub dst: String,
}

pub fn run(args: CompressArgs) {
    println!("[compress] {} -> {}", args.src, args.dst);
}