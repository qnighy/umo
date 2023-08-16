// SIR -- Sequential Intermediate Representation

use std::sync::Arc;

use crate::cctx::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProgramUnit {
    pub functions: Vec<Function>,
}

impl ProgramUnit {
    pub fn new(functions: Vec<Function>) -> Self {
        Self { functions }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    /// Number of arguments, Must be <= num_vars
    pub num_args: usize,
    /// Number of local variables, including args
    pub num_vars: usize,
    pub body: Vec<BasicBlock>,
}

impl Function {
    pub fn new(num_args: usize, num_vars: usize, body: Vec<BasicBlock>) -> Self {
        Self {
            num_args,
            num_vars,
            body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BasicBlock {
    pub insts: Vec<Inst>,
}

impl BasicBlock {
    pub fn new(insts: Vec<Inst>) -> Self {
        assert!(insts.len() > 0);
        assert!(insts[..insts.len() - 1].iter().all(|i| i.kind.is_middle()));
        assert!(insts.last().unwrap().kind.is_tail());
        Self { insts }
    }

    pub fn new_partial(insts: Vec<Inst>) -> Self {
        assert!(insts.iter().all(|i| i.kind.is_middle()));
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
    Return {
        rhs: usize,
    },
    Copy {
        lhs: usize,
        rhs: usize,
    },
    Drop {
        rhs: usize,
    },
    Literal {
        lhs: usize,
        value: Literal,
    },
    PushArg {
        value_ref: usize,
    },
    #[allow(unused)] // TODO: remove it later
    Call {
        lhs: usize,
        callee: usize,
    },
    CallBuiltin {
        lhs: usize,
        builtin: BuiltinKind,
    },
}

impl InstKind {
    pub fn is_tail(&self) -> bool {
        match self {
            InstKind::Jump { .. } | InstKind::Branch { .. } | InstKind::Return { .. } => true,
            InstKind::Copy { .. }
            | InstKind::Drop { .. }
            | InstKind::Literal { .. }
            | InstKind::PushArg { .. }
            | InstKind::Call { .. }
            | InstKind::CallBuiltin { .. } => false,
        }
    }
    pub fn is_middle(&self) -> bool {
        !self.is_tail()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    Unit,
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
    use crate::sir::{BasicBlock, Function, Inst, ProgramUnit};
    use crate::testing::SeqGen;

    pub trait ProgramUnitTestingExt {
        fn describe<FS, F>(f: F) -> Self
        where
            FS: SeqGen,
            F: FnOnce(&mut ProgramUnitDescriber, FS);

        fn simple(function: Function) -> Self;
    }

    impl ProgramUnitTestingExt for ProgramUnit {
        fn describe<FS, F>(f: F) -> Self
        where
            FS: SeqGen,
            F: FnOnce(&mut ProgramUnitDescriber, FS),
        {
            let mut desc = ProgramUnitDescriber {
                program_unit: ProgramUnit {
                    functions: vec![
                        Function {
                            num_args: 0,
                            num_vars: 0,
                            body: vec![]
                        };
                        FS::size()
                    ],
                },
            };
            f(&mut desc, FS::seq());
            desc.program_unit
        }

        fn simple(function: Function) -> Self {
            Self {
                functions: vec![function],
            }
        }
    }

    #[derive(Debug)]
    pub struct ProgramUnitDescriber {
        program_unit: ProgramUnit,
    }

    impl ProgramUnitDescriber {
        #[allow(unused)] // TODO: remove it later
        pub fn function(&mut self, function_id: usize, function: Function) -> &mut Self {
            self.program_unit.functions[function_id] = function;
            self
        }
    }

    pub trait FunctionTestingExt {
        fn describe<VS, BS, F>(num_args: usize, f: F) -> Self
        where
            VS: SeqGen,
            BS: SeqGen,
            F: FnOnce(&mut FunctionDescriber, VS, BS);

        fn simple<VS, F>(num_args: usize, f: F) -> Self
        where
            VS: SeqGen,
            F: FnOnce(VS) -> Vec<Inst>;
    }

    impl FunctionTestingExt for Function {
        fn describe<VS, BS, F>(num_args: usize, f: F) -> Self
        where
            VS: SeqGen,
            BS: SeqGen,
            F: FnOnce(&mut FunctionDescriber, VS, BS),
        {
            let mut desc = FunctionDescriber {
                function: Self::new(
                    num_args,
                    VS::size(),
                    vec![BasicBlock::default(); BS::size()],
                ),
            };
            f(&mut desc, VS::seq(), BS::seq());
            desc.function
        }

        fn simple<VS, F>(num_args: usize, f: F) -> Self
        where
            VS: SeqGen,
            F: FnOnce(VS) -> Vec<Inst>,
        {
            Self::new(num_args, VS::size(), vec![BasicBlock::new(f(VS::seq()))])
        }
    }

    #[derive(Debug)]
    pub struct FunctionDescriber {
        function: Function,
    }

    impl FunctionDescriber {
        pub fn block(&mut self, bb_id: usize, insts: Vec<Inst>) -> &mut Self {
            self.function.body[bb_id] = BasicBlock::new(insts);
            self
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
        pub fn return_(rhs: usize) -> Inst {
            Inst::new(InstKind::Return { rhs })
        }
        pub fn copy(lhs: usize, rhs: usize) -> Inst {
            Inst::new(InstKind::Copy { lhs, rhs })
        }
        pub fn drop(rhs: usize) -> Inst {
            Inst::new(InstKind::Drop { rhs })
        }
        pub fn unit_literal(lhs: usize) -> Inst {
            Inst::new(InstKind::Literal {
                lhs,
                value: Literal::Unit,
            })
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
        pub fn call(lhs: usize, function: usize) -> Inst {
            Inst::new(InstKind::Call {
                lhs,
                callee: function,
            })
        }
        pub fn add(lhs: usize) -> Inst {
            Inst::new(InstKind::CallBuiltin {
                lhs,
                builtin: BuiltinKind::Add,
            })
        }
        pub fn lt(lhs: usize) -> Inst {
            Inst::new(InstKind::CallBuiltin {
                lhs,
                builtin: BuiltinKind::Lt,
            })
        }
        pub fn puts(dummy_lhs: usize) -> Inst {
            Inst::new(InstKind::CallBuiltin {
                lhs: dummy_lhs,
                builtin: BuiltinKind::Puts,
            })
        }
        pub fn puti(dummy_lhs: usize) -> Inst {
            Inst::new(InstKind::CallBuiltin {
                lhs: dummy_lhs,
                builtin: BuiltinKind::Puti,
            })
        }
    }
}
