use crate::eval::BuiltinKind;

pub(crate) struct Program {
    pub(crate) functions: Vec<FunDef>,
    pub(crate) main: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FunDef {
    pub(crate) num_args: usize,
    pub(crate) num_locals: usize,
    pub(crate) body: Vec<BasicBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BasicBlock {
    pub(crate) middle: Vec<MInst>,
    pub(crate) tail: TInst,
}

/// Middle instruction
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MInst {
    Read(usize),
    Write(usize),
    CCall(/** num_args */ usize),
    Closure(/** num_capture */ usize, /** function id */ usize),
    Int(i32),
    Arr(/** len */ usize),
    Builtin(BuiltinKind),
}

/// Tail instruction
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TInst {
    Ret,
    Jump(/** target */ usize),
    JumpIf(/** then-target */ usize, /** else-target */ usize),
    TCCall,
}
