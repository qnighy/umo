// SIR -- Sequential Intermediate Representation

use std::sync::Arc;

// Define BasicBlock
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicBlock {
    // To be hoisted to FunDef
    pub num_vars: usize,
    pub insts: Vec<Inst>,
}

impl BasicBlock {
    pub fn new(num_vars: usize, insts: Vec<Inst>) -> Self {
        Self { num_vars, insts }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Inst {
    pub id: usize,
    pub kind: InstKind,
}

impl Inst {
    pub fn new(kind: InstKind) -> Self {
        Self { id: 0, kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstKind {
    Copy { lhs: usize, rhs: usize },
    StringLiteral { lhs: usize, value: Arc<String> },
    PushArg { value_ref: usize },
    Puts,
}
