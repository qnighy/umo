// SIR -- Sequential Intermediate Representation

use std::sync::Arc;

use crate::cctx::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    pub num_vars: usize,
    // pub num_args: usize,
    pub body: Vec<BasicBlock>,
}

impl Function {
    pub fn new(num_vars: usize, body: Vec<BasicBlock>) -> Self {
        Self { num_vars, body }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicBlock {
    pub insts: Vec<Inst>,
}

impl BasicBlock {
    pub fn new(insts: Vec<Inst>) -> Self {
        Self { insts }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Inst {
    pub id: Id,
    pub kind: InstKind,
}

impl Inst {
    pub fn new(kind: InstKind) -> Self {
        Self {
            id: Id::default(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstKind {
    #[allow(unused)] // TODO: remove it later
    Jump {
        target: usize,
    },
    #[allow(unused)] // TODO: remove it later
    Branch {
        cond: usize,
        branch_then: usize,
        branch_else: usize,
    },
    Copy {
        lhs: usize,
        rhs: usize,
    },
    Literal {
        lhs: usize,
        value: Literal,
    },
    PushArg {
        value_ref: usize,
    },
    CallBuiltin {
        lhs: Option<usize>,
        builtin: BuiltinKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    // TODO: use BigInt
    Integer(i32),
    #[allow(unused)] // TODO: remove it later
    Bool(bool),
    String(Arc<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinKind {
    Add,
    #[allow(unused)] // TODO: remove it later
    Lt,
    Puts,
    Puti,
}

#[cfg(test)]
pub mod testing {
    use crate::sir::{BasicBlock, Function};
    use crate::testing::SeqGen;

    pub trait FunctionTestingExt {
        fn describe<T, F>(f: F) -> Self
        where
            T: SeqGen,
            F: FnOnce(T) -> Vec<BasicBlock>;
    }

    impl FunctionTestingExt for Function {
        fn describe<T, F>(f: F) -> Self
        where
            T: SeqGen,
            F: FnOnce(T) -> Vec<BasicBlock>,
        {
            let body = f(T::seq());
            Self::new(T::size(), body)
        }
    }

    pub mod insts {
        use crate::sir::{BuiltinKind, Inst, InstKind, Literal};
        use std::sync::Arc;
        pub fn jump(target: usize) -> Inst {
            Inst::new(InstKind::Jump { target })
        }
        pub fn branch(cond: usize, branch_then: usize, branch_else: usize) -> Inst {
            Inst::new(InstKind::Branch {
                cond,
                branch_then,
                branch_else,
            })
        }
        pub fn copy(lhs: usize, rhs: usize) -> Inst {
            Inst::new(InstKind::Copy { lhs, rhs })
        }
        pub fn integer_literal(lhs: usize, value: i32) -> Inst {
            Inst::new(InstKind::Literal {
                lhs,
                value: Literal::Integer(value),
            })
        }
        pub fn bool_literal(lhs: usize, value: bool) -> Inst {
            Inst::new(InstKind::Literal {
                lhs,
                value: Literal::Bool(value),
            })
        }
        pub fn string_literal(lhs: usize, value: &str) -> Inst {
            Inst::new(InstKind::Literal {
                lhs,
                value: Literal::String(Arc::new(value.to_owned())),
            })
        }
        pub fn push_arg(value_ref: usize) -> Inst {
            Inst::new(InstKind::PushArg { value_ref })
        }
        pub fn add(lhs: usize) -> Inst {
            Inst::new(InstKind::CallBuiltin {
                lhs: Some(lhs),
                builtin: BuiltinKind::Add,
            })
        }
        pub fn lt(lhs: usize) -> Inst {
            Inst::new(InstKind::CallBuiltin {
                lhs: Some(lhs),
                builtin: BuiltinKind::Lt,
            })
        }
        pub fn puts() -> Inst {
            Inst::new(InstKind::CallBuiltin {
                lhs: None,
                builtin: BuiltinKind::Puts,
            })
        }
        pub fn puti() -> Inst {
            Inst::new(InstKind::CallBuiltin {
                lhs: None,
                builtin: BuiltinKind::Puti,
            })
        }
    }
}
