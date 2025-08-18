use clap::{Parser, Subcommand};

pub mod find;
pub mod rm;
pub mod compress;
pub mod mv;
pub mod flow;

#[derive(Parser, Debug)]
#[command(name = "file-rs")]
#[command(version, about = "A fast and safe file organizer and cleaner built with Rust.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Compress(compress::CompressArgs),
    Find(find::FindArgs),
    Rm(rm::RmArgs),
    #[command(alias = "move")]
    Mv(mv::MoveArgs),
    Flow(flow::FlowArgs)
}

pub fn run(cli: Cli) {
    match cli.command {
        Commands::Find(args) => find::run(args),
        Commands::Rm(args) => rm::run(args),
        Commands::Compress(args) => compress::run(args),
        Commands::Mv(args) => mv::run(args),
        Commands::Flow(args) => flow::run(args),
    }
}
