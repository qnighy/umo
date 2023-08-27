// SIR -- Sequential Intermediate Representation

use std::fmt;
use std::sync::Arc;

use crate::cctx::Id;
use crate::util::SeqInit;

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

    pub fn describe<const NV: usize, const NB: usize, F>(num_args: usize, f: F) -> Self
    where
        F: FnOnce([usize; NV], [usize; NB]) -> Vec<(usize, Vec<Inst>)>,
    {
        let bbs_indexed = f(SeqInit::seq(), SeqInit::seq());
        let mut function = Self::new(num_args, NV, vec![BasicBlock::default(); NB]);
        for (i, (bb_id, insts)) in bbs_indexed.into_iter().enumerate() {
            assert_eq!(
                bb_id,
                i,
                "The {}th bb_id should be {}, got {}",
                i + 1,
                i,
                bb_id
            );
            function.body[i] = BasicBlock::new(insts);
        }
        function
    }

    pub fn simple<const NV: usize, F>(num_args: usize, f: F) -> Self
    where
        F: FnOnce([usize; NV]) -> Vec<Inst>,
    {
        Self::new(num_args, NV, vec![BasicBlock::new(f(SeqInit::seq()))])
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct BasicBlock {
    pub insts: Vec<Inst>,
}

impl BasicBlock {
    pub fn new<A>(insts: A) -> Self
    where
        A: Into<Vec<Inst>>,
    {
        Self {
            insts: insts.into(),
        }
    }
}

impl fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BasicBlock::new").field(&self.insts).finish()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
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
    pub fn with_id(self, id: Id) -> Self {
        Self { id, ..self }
    }

    pub fn jump(target: usize) -> Self {
        Self::new(InstKind::Jump { target })
    }
    pub fn branch(cond: usize, branch_then: usize, branch_else: usize) -> Self {
        Self::new(InstKind::Branch {
            cond,
            branch_then,
            branch_else,
        })
    }
    pub fn return_(rhs: usize) -> Self {
        Self::new(InstKind::Return { rhs })
    }
    pub fn copy(lhs: usize, rhs: usize) -> Self {
        Self::new(InstKind::Copy { lhs, rhs })
    }
    pub fn drop(rhs: usize) -> Self {
        Self::new(InstKind::Drop { rhs })
    }
    pub fn literal<L: Into<Literal>>(lhs: usize, value: L) -> Self {
        Self::new(InstKind::Literal {
            lhs,
            value: value.into(),
        })
    }
    pub fn closure(lhs: usize, function_id: usize) -> Self {
        Self::new(InstKind::Closure { lhs, function_id })
    }
    pub fn builtin(lhs: usize, builtin: BuiltinKind) -> Self {
        Self::new(InstKind::Builtin { lhs, builtin })
    }
    pub fn push_arg(value_ref: usize) -> Self {
        Self::new(InstKind::PushArg { value_ref })
    }
    pub fn call(lhs: usize, callee: usize) -> Self {
        Self::new(InstKind::Call { lhs, callee })
    }
}

impl fmt::Debug for Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            InstKind::Jump { target } => f.debug_tuple("Inst::jump").field(target).finish()?,
            InstKind::Branch {
                cond,
                branch_then,
                branch_else,
            } => f
                .debug_tuple("Inst::branch")
                .field(cond)
                .field(branch_then)
                .field(branch_else)
                .finish()?,
            InstKind::Return { rhs } => f.debug_tuple("Inst::return_").field(rhs).finish()?,
            InstKind::Copy { lhs, rhs } => {
                f.debug_tuple("Inst::copy").field(lhs).field(rhs).finish()?
            }
            InstKind::Drop { rhs } => f.debug_tuple("Inst::drop").field(rhs).finish()?,
            InstKind::Literal { lhs, value } => f
                .debug_tuple("Inst::literal")
                .field(lhs)
                .field(&value.debug_inner())
                .finish()?,
            InstKind::Closure { lhs, function_id } => f
                .debug_tuple("Inst::closure")
                .field(lhs)
                .field(function_id)
                .finish()?,
            InstKind::Builtin { lhs, builtin } => f
                .debug_tuple("Inst::builtin")
                .field(lhs)
                .field(builtin)
                .finish()?,
            InstKind::PushArg { value_ref } => {
                f.debug_tuple("Inst::push_arg").field(value_ref).finish()?
            }
            InstKind::Call { lhs, callee } => f
                .debug_tuple("Inst::call")
                .field(lhs)
                .field(callee)
                .finish()?,
        }
        if !self.id.is_dummy() {
            f.debug_tuple(".with_id").field(&self.id).finish()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstKind {
    Jump {
        target: usize,
    },
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
    Closure {
        lhs: usize,
        function_id: usize,
    },
    Builtin {
        lhs: usize,
        builtin: BuiltinKind,
    },
    PushArg {
        value_ref: usize,
    },
    Call {
        lhs: usize,
        callee: usize,
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
            | InstKind::Closure { .. }
            | InstKind::Builtin { .. }
            | InstKind::Call { .. } => false,
        }
    }
    pub fn is_middle(&self) -> bool {
        !self.is_tail()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    Unit,
    // TODO: use BigInt
    Integer(i32),
    Bool(bool),
    String(Arc<String>),
}

impl From<()> for Literal {
    fn from(_: ()) -> Self {
        Self::Unit
    }
}
impl From<i32> for Literal {
    fn from(i: i32) -> Self {
        Self::Integer(i)
    }
}
impl From<bool> for Literal {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}
impl From<&str> for Literal {
    fn from(s: &str) -> Self {
        Self::String(Arc::new(s.to_owned()))
    }
}

impl Literal {
    fn debug_inner(&self) -> impl fmt::Debug + '_ {
        struct D<'a>(&'a Literal);
        return D(self);

        impl fmt::Debug for D<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.0 {
                    Literal::Unit => write!(f, "()"),
                    Literal::Integer(i) => write!(f, "{:?}", i),
                    Literal::Bool(b) => write!(f, "{:?}", b),
                    Literal::String(s) => write!(f, "{:?}", s),
                }
            }
        }
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Unit => write!(f, "Literal::from(())"),
            Literal::Integer(i) => write!(f, "Literal::from({:?})", i),
            Literal::Bool(b) => write!(f, "Literal::from({:?})", b),
            Literal::String(s) => write!(f, "Literal::from({:?})", s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinKind {
    Add,
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
        pub fn function(&mut self, function_id: usize, function: Function) -> &mut Self {
            self.program_unit.functions[function_id] = function;
            self
        }
    }
}
