// SIR -- Sequential Intermediate Representation

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inst {
    Puts,
}

pub fn eval(inst: &Inst) {
    match inst {
        Inst::Puts => println!("Hello, world!"),
    }
}
