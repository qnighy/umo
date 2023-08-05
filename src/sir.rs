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

#[cfg(test)]
pub mod testing {
    use crate::sir::{BasicBlock, Inst};
    use crate::testing::SeqGen;

    pub trait BasicBlockTestingExt {
        fn describe<T, F>(f: F) -> Self
        where
            T: SeqGen,
            F: FnOnce(T) -> Vec<Inst>;
    }

    impl BasicBlockTestingExt for BasicBlock {
        fn describe<T, F>(f: F) -> Self
        where
            T: SeqGen,
            F: FnOnce(T) -> Vec<Inst>,
        {
            let insts = f(T::seq());
            Self::new(T::size(), insts)
        }
    }

    pub mod insts {
        use crate::sir::{Inst, InstKind};
        pub fn copy(lhs: usize, rhs: usize) -> Inst {
            Inst::new(InstKind::Copy { lhs, rhs })
        }
        pub fn string_literal(lhs: usize, value: &str) -> Inst {
            Inst::new(InstKind::StringLiteral {
                lhs,
                value: std::sync::Arc::new(value.to_owned()),
            })
        }
        pub fn push_arg(value_ref: usize) -> Inst {
            Inst::new(InstKind::PushArg { value_ref })
        }
        pub fn puts() -> Inst {
            Inst::new(InstKind::Puts)
        }
    }
}
