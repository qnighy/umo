use std::fs;
use std::path::Path;

pub mod old;
mod sir;

pub fn run(source_path: &Path) {
    let source = fs::read_to_string(source_path).unwrap();
    if source == "use lang::\"0.0.1\";\nputs(\"Hello, world!\");\n" {
        sir::eval(&sir::Inst::Puts);
    } else {
        todo!("Proper parsing and execution");
    }
}
