// SIR -- Sequential Intermediate Representation

use std::fmt;
use std::sync::Arc;

use bit_set::BitSet;

use crate::util::debug_utils::{debug_with, debug_with_display, PDebug, PDebugExt};
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
            f.debug_tuple("ProgramUnit::describe")
                .field(&debug_with(|f| {
                    let functions = (0..self.functions.len())
                        .map(|i| format!("f{}", i))
                        .collect::<Vec<_>>();

                    f.write_str("|")?;
                    f.debug_list()
                        .entries(functions.iter().map(|f| debug_with_display(f)))
                        .finish()?;
                    f.write_str("| ")?;
                    f.debug_list()
                        .entries(self.functions.iter().enumerate().map(|(i, function)| {
                            (
                                i,
                                function.debug_with(FunctionDebugParams {
                                    functions: &functions,
                                }),
                            )
                        }))
                        .finish()?;
                    Ok(())
                }))
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
        F: FnOnce([usize; NV], [usize; NB]) -> Vec<(usize, BasicBlock)>,
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
            function.body[i] = insts;
        }
        function
    }

    pub fn simple<const NV: usize, F>(num_args: usize, f: F) -> Self
    where
        F: FnOnce([usize; NV]) -> BasicBlock,
    {
        Self::new(num_args, NV, vec![f(SeqInit::seq())])
    }
}

#[derive(Debug, Clone, Copy)]
struct FunctionDebugParams<'a> {
    functions: &'a [String],
}

impl<'a> PDebug<FunctionDebugParams<'a>> for Function {
    fn pfmt(
        &self,
        f: &mut fmt::Formatter<'_>,
        FunctionDebugParams { functions }: FunctionDebugParams<'a>,
    ) -> fmt::Result {
        if self.body.len() == 1 {
            f.debug_tuple("Function::simple")
                .field(&self.num_args)
                .field(&debug_with(|f| {
                    let vars = (0..self.num_vars)
                        .map(|i| format!("v{}", i))
                        .collect::<Vec<_>>();
                    f.write_str("|")?;
                    f.debug_list()
                        .entries(vars.iter().map(|v| debug_with_display(v)))
                        .finish()?;
                    f.write_str("| ")?;
                    self.body[0].pfmt(
                        f,
                        InstDebugParams {
                            vars: &vars,
                            blocks: &[],
                            functions,
                        },
                    )?;
                    Ok(())
                }))
                .finish()
        } else {
            f.debug_tuple("Function::describe")
                .field(&self.num_args)
                .field(&debug_with(|f| {
                    let vars = (0..self.num_vars)
                        .map(|i| format!("v{}", i))
                        .collect::<Vec<_>>();
                    let blocks = (0..self.body.len())
                        .map(|i| format!("bb{}", i))
                        .collect::<Vec<_>>();

                    f.write_str("|")?;
                    f.debug_list()
                        .entries(vars.iter().map(|v| debug_with_display(v)))
                        .finish()?;
                    f.write_str(", ")?;
                    f.debug_list()
                        .entries(blocks.iter().map(|v| debug_with_display(v)))
                        .finish()?;
                    f.write_str("| ")?;
                    f.debug_list()
                        .entries(self.body.iter().enumerate().map(|(i, bb)| {
                            (
                                i,
                                bb.debug_with(InstDebugParams {
                                    vars: &vars,
                                    blocks: &blocks,
                                    functions,
                                }),
                            )
                        }))
                        .finish()?;
                    Ok(())
                }))
                .finish()
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pfmt(f, FunctionDebugParams { functions: &[] })
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct BasicBlock {
    pub insts: Vec<Inst>,
    // live_out can be cheaply computed from the last instruction
    /// Variables that are live at the beginning of this block
    pub live_in: Option<BitSet<usize>>,
}

impl BasicBlock {
    pub fn new<A>(insts: A) -> Self
    where
        A: Into<Vec<Inst>>,
    {
        Self {
            insts: insts.into(),
            live_in: None,
        }
    }
    pub fn with_live_in(mut self, live_in: BitSet<usize>) -> Self {
        self.live_in = Some(live_in);
        self
    }
}

impl<'a> PDebug<InstDebugParams<'a>> for BasicBlock {
    fn pfmt(
        &self,
        f: &mut fmt::Formatter<'_>,
        InstDebugParams {
            vars,
            blocks,
            functions,
        }: InstDebugParams<'a>,
    ) -> fmt::Result {
        f.debug_tuple("BasicBlock::new")
            .field(&self.insts.debug_with(InstDebugParams {
                vars,
                blocks,
                functions,
            }))
            .finish()?;
        if let Some(live_in) = &self.live_in {
            f.debug_tuple(".with_live_in")
                .field(&debug_bit_set(live_in, vars))
                .finish()?;
        }
        Ok(())
    }
}

impl fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pfmt(
            f,
            InstDebugParams {
                vars: &[],
                blocks: &[],
                functions: &[],
            },
        )
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Inst {
    pub kind: InstKind,
    // live_in can be cheaply computed from live_out
    /// Variables that are live after this instruction
    pub live_out: Option<BitSet<usize>>,
}

impl Inst {
    pub fn new(kind: InstKind) -> Self {
        Self {
            kind,
            live_out: None,
        }
    }
    pub fn with_live_out(mut self, live_out: BitSet<usize>) -> Self {
        self.live_out = Some(live_out);
        self
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

impl<'a> PDebug<InstDebugParams<'a>> for Inst {
    fn pfmt(
        &self,
        f: &mut fmt::Formatter<'_>,
        InstDebugParams {
            vars,
            blocks,
            functions,
        }: InstDebugParams<'a>,
    ) -> fmt::Result {
        match &self.kind {
            InstKind::Jump { target } => f
                .debug_tuple("Inst::jump")
                .field(&debug_block(*target, blocks))
                .finish()?,
            InstKind::Branch {
                cond,
                branch_then,
                branch_else,
            } => f
                .debug_tuple("Inst::branch")
                .field(&debug_var(*cond, vars))
                .field(&debug_block(*branch_then, blocks))
                .field(&debug_block(*branch_else, blocks))
                .finish()?,
            InstKind::Return { rhs } => f
                .debug_tuple("Inst::return_")
                .field(&debug_var(*rhs, vars))
                .finish()?,
            InstKind::Copy { lhs, rhs } => f
                .debug_tuple("Inst::copy")
                .field(&debug_var(*lhs, vars))
                .field(&debug_var(*rhs, vars))
                .finish()?,
            InstKind::Drop { rhs } => f
                .debug_tuple("Inst::drop")
                .field(&debug_var(*rhs, vars))
                .finish()?,
            InstKind::Literal { lhs, value } => f
                .debug_tuple("Inst::literal")
                .field(&debug_var(*lhs, vars))
                .field(&value.debug_inner())
                .finish()?,
            InstKind::Closure { lhs, function_id } => f
                .debug_tuple("Inst::closure")
                .field(&debug_var(*lhs, vars))
                .field(&debug_function(*function_id, functions))
                .finish()?,
            InstKind::Builtin { lhs, builtin } => f
                .debug_tuple("Inst::builtin")
                .field(&debug_var(*lhs, vars))
                .field(builtin)
                .finish()?,
            InstKind::PushArg { value_ref } => f
                .debug_tuple("Inst::push_arg")
                .field(&debug_var(*value_ref, vars))
                .finish()?,
            InstKind::Call { lhs, callee } => f
                .debug_tuple("Inst::call")
                .field(&debug_var(*lhs, vars))
                .field(&debug_var(*callee, vars))
                .finish()?,
        }
        if let Some(live_out) = &self.live_out {
            f.debug_tuple(".with_live_out")
                .field(&debug_bit_set(live_out, vars))
                .finish()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct InstDebugParams<'a> {
    vars: &'a [String],
    blocks: &'a [String],
    functions: &'a [String],
}

impl fmt::Debug for Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pfmt(
            f,
            InstDebugParams {
                vars: &[],
                blocks: &[],
                functions: &[],
            },
        )
    }
}

fn debug_bit_set<'a>(bit_set: &'a BitSet<usize>, vars: &'a [String]) -> impl fmt::Debug + 'a {
    debug_with(|f| {
        f.debug_list()
            .entries(bit_set.iter().map(|v| debug_var(v, vars)))
            .finish()?;
        f.write_str(".into_iter()")?;
        f.write_str(".collect()")?;
        Ok(())
    })
}

fn debug_var<'a>(var: usize, vars: &'a [String]) -> impl fmt::Debug + 'a {
    debug_with(move |f| {
        if let Some(var_name) = vars.get(var) {
            write!(f, "{}", var_name)
        } else {
            write!(f, "{}", var)
        }
    })
}

fn debug_block<'a>(block: usize, blocks: &'a [String]) -> impl fmt::Debug + 'a {
    debug_with(move |f| {
        if let Some(block_name) = blocks.get(block) {
            write!(f, "{}", block_name)
        } else {
            write!(f, "{}", block)
        }
    })
}

fn debug_function<'a>(function: usize, functions: &'a [String]) -> impl fmt::Debug + 'a {
    debug_with(move |f| {
        if let Some(function_name) = functions.get(function) {
            write!(f, "{}", function_name)
        } else {
            write!(f, "{}", function)
        }
    })
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
        debug_with(move |f| match self {
            Literal::Unit => write!(f, "()"),
            Literal::Integer(i) => write!(f, "{:?}", i),
            Literal::Bool(b) => write!(f, "{:?}", b),
            Literal::String(s) => write!(f, "{:?}", s),
        })
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
