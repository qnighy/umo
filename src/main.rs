use std::fs;
use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    source: PathBuf,
}

fn main() {
    let args = Args::parse();
    let source = fs::read_to_string(&args.source).unwrap();
    if source == "use lang::\"0.0.1\";\nputs(\"Hello, world!\");\n" {
        println!("Hello, world!");
    } else {
        todo!("Proper parsing and execution");
    }
}
