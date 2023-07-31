use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    source: PathBuf,
}

fn main() {
    let args = Args::parse();
    umo::run(&args.source);
}
