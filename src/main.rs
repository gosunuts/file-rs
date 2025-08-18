mod cmd;

use clap::Parser;

fn main() {
    let cli = cmd::Cli::parse();
    cmd::run(cli);
}