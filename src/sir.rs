// SIR -- Sequential Intermediate Representation

use std::fmt;
use std::sync::Arc;

use crate::cctx::Id;
use crate::util::SeqInit;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ProgramUnit {
    pub functions: Vec<Function>,
}

impl ProgramUnit {
    pub fn new(functions: Vec<Function>) -> Self {
        Self { functions }
    }

    pub fn describe<const NF: usize, F>(f: F) -> Self
    where
        F: FnOnce([usize; NF]) -> Vec<(usize, Function)>,
    {
        let functions_indexed = f(SeqInit::seq());
        for (i, &(function_id, _)) in functions_indexed.iter().enumerate() {
            assert_eq!(
                function_id,
                i,
                "The {}th function_id should be {}, got {}",
                i + 1,
                i,
                function_id
            );
        }
        let functions = functions_indexed
            .into_iter()
            .map(|(_, function)| function)
            .collect::<Vec<_>>();
        Self::new(functions)
    }

    pub fn simple(function: Function) -> Self {
        Self {
            functions: vec![function],
        }
    }
}

impl fmt::Debug for ProgramUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.functions.len() == 1 {
            f.debug_tuple("ProgramUnit::simple")
                .field(&self.functions[0])
                .finish()
        } else {
            struct ClosureWrapper<'a>(&'a ProgramUnit);
            impl fmt::Debug for ClosureWrapper<'_> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let functions = (0..self.0.functions.len())
                        .map(|i| format!("f{}", i))
                        .collect::<Vec<_>>();

                    f.write_str("|")?;
                    f.debug_list()
                        .entries(functions.iter().map(|f| IdentPrinter(f)))
                        .finish()?;
                    f.write_str("| ")?;
                    f.debug_list()
                        .entries(self.0.functions.iter().enumerate())
                        .finish()?;
                    Ok(())
                }
            }

            f.debug_tuple("ProgramUnit::describe")
                .field(&ClosureWrapper(self))
                .finish()
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
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

    fn fmt_debug_with(&self, f: &mut fmt::Formatter<'_>, functions: &[String]) -> fmt::Result {
        if self.body.len() == 1 {
            struct SimpleClosureWrapper<'a>(&'a Function, &'a [String]);
            impl fmt::Debug for SimpleClosureWrapper<'_> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let functions = self.1;
                    let vars = (0..self.0.num_vars)
                        .map(|i| format!("v{}", i))
                        .collect::<Vec<_>>();
                    f.write_str("|")?;
                    f.debug_list()
                        .entries(vars.iter().map(|v| IdentPrinter(v)))
                        .finish()?;
                    f.write_str("| ")?;
                    f.debug_list()
                        .entries(
                            &self.0.body[0]
                                .insts
                                .iter()
                                .map(|inst| inst.debug_with(&vars, &[], functions))
                                .collect::<Vec<_>>(),
                        )
                        .finish()?;
                    Ok(())
                }
            }

            f.debug_tuple("Function::simple")
                .field(&self.num_args)
                .field(&SimpleClosureWrapper(self, functions))
                .finish()
        } else {
            struct ClosureWrapper<'a>(&'a Function, &'a [String]);
            impl fmt::Debug for ClosureWrapper<'_> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let functions = self.1;
                    let vars = (0..self.0.num_vars)
                        .map(|i| format!("v{}", i))
                        .collect::<Vec<_>>();
                    let blocks = (0..self.0.body.len())
                        .map(|i| format!("bb{}", i))
                        .collect::<Vec<_>>();

                    f.write_str("|")?;
                    f.debug_list()
                        .entries(vars.iter().map(|v| IdentPrinter(v)))
                        .finish()?;
                    f.write_str(", ")?;
                    f.debug_list()
                        .entries(blocks.iter().map(|v| IdentPrinter(v)))
                        .finish()?;
                    f.write_str("| ")?;
                    f.debug_list()
                        .entries(self.0.body.iter().enumerate().map(|(i, bb)| {
                            (
                                i,
                                bb.insts
                                    .iter()
                                    .map(|inst| inst.debug_with(&vars, &blocks, functions))
                                    .collect::<Vec<_>>(),
                            )
                        }))
                        .finish()?;
                    Ok(())
                }
            }

            f.debug_tuple("Function::describe")
                .field(&self.num_args)
                .field(&ClosureWrapper(self, functions))
                .finish()
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_debug_with(f, &[])
    }
}

struct IdentPrinter<'a>(&'a str);
impl fmt::Debug for IdentPrinter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
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

    fn debug_with<'a>(
        &'a self,
        vars: &'a [String],
        blocks: &'a [String],
        functions: &'a [String],
    ) -> impl fmt::Debug + 'a {
        struct D<'a>(&'a Inst, &'a [String], &'a [String], &'a [String]);
        return D(self, vars, blocks, functions);

        impl fmt::Debug for D<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt_debug_with(f, self.1, self.2, self.3)
            }
        }
    }

    fn fmt_debug_with(
        &self,
        f: &mut fmt::Formatter<'_>,
        vars: &[String],
        blocks: &[String],
        functions: &[String],
    ) -> fmt::Result {
        match &self.kind {
            InstKind::Jump { target } => f
                .debug_tuple("Inst::jump")
                .field(&BlockDebugWrapper(*target, blocks))
                .finish()?,
            InstKind::Branch {
                cond,
                branch_then,
                branch_else,
            } => f
                .debug_tuple("Inst::branch")
                .field(&VarDebugWrapper(*cond, vars))
                .field(&BlockDebugWrapper(*branch_then, blocks))
                .field(&BlockDebugWrapper(*branch_else, blocks))
                .finish()?,
            InstKind::Return { rhs } => f
                .debug_tuple("Inst::return_")
                .field(&VarDebugWrapper(*rhs, vars))
                .finish()?,
            InstKind::Copy { lhs, rhs } => f
                .debug_tuple("Inst::copy")
                .field(&VarDebugWrapper(*lhs, vars))
                .field(&VarDebugWrapper(*rhs, vars))
                .finish()?,
            InstKind::Drop { rhs } => f
                .debug_tuple("Inst::drop")
                .field(&VarDebugWrapper(*rhs, vars))
                .finish()?,
            InstKind::Literal { lhs, value } => f
                .debug_tuple("Inst::literal")
                .field(&VarDebugWrapper(*lhs, vars))
                .field(&value.debug_inner())
                .finish()?,
            InstKind::Closure { lhs, function_id } => f
                .debug_tuple("Inst::closure")
                .field(&VarDebugWrapper(*lhs, vars))
                .field(&FunctionDebugWrapper(*function_id, functions))
                .finish()?,
            InstKind::Builtin { lhs, builtin } => f
                .debug_tuple("Inst::builtin")
                .field(&VarDebugWrapper(*lhs, vars))
                .field(builtin)
                .finish()?,
            InstKind::PushArg { value_ref } => f
                .debug_tuple("Inst::push_arg")
                .field(&VarDebugWrapper(*value_ref, vars))
                .finish()?,
            InstKind::Call { lhs, callee } => f
                .debug_tuple("Inst::call")
                .field(&VarDebugWrapper(*lhs, vars))
                .field(&VarDebugWrapper(*callee, vars))
                .finish()?,
        }
        if !self.id.is_dummy() {
            f.debug_tuple(".with_id").field(&self.id).finish()?;
        }
        Ok(())
    }
}

impl fmt::Debug for Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_debug_with(f, &[], &[], &[])
    }
}

struct VarDebugWrapper<'a>(usize, &'a [String]);
impl fmt::Debug for VarDebugWrapper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(var_name) = self.1.get(self.0) {
            write!(f, "{}", var_name)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

struct BlockDebugWrapper<'a>(usize, &'a [String]);
impl fmt::Debug for BlockDebugWrapper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(block_name) = self.1.get(self.0) {
            write!(f, "{}", block_name)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

struct FunctionDebugWrapper<'a>(usize, &'a [String]);
impl fmt::Debug for FunctionDebugWrapper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(function_name) = self.1.get(self.0) {
            write!(f, "{}", function_name)
        } else {
            write!(f, "{}", self.0)
        }
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
