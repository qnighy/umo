use std::fmt;

use thiserror::Error;

use crate::sir::{BasicBlock, Function, Inst, ProgramUnit};

#[derive(Debug, Error)]
pub enum SirValidationError {
    #[error("excess number of arguments at {pos}")]
    ExcessNumArgs { pos: SirPosition },
    #[error("expected tail instruction at {pos}")]
    ExpectedTailInstruction { pos: SirPosition },
    #[error("unexpected tail instruction at {pos}")]
    UnexpectedTailInstruction { pos: SirPosition },
    #[error("invalid variable id at {pos}")]
    InvalidVariableId { pos: SirPosition },
    #[error("invalid jump/branch target at {pos}")]
    InvalidTargetBlock { pos: SirPosition },
    #[error("invalid function id at {pos}")]
    InvalidFunctionId { pos: SirPosition },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SirPosition {
    pub function_id: usize,
    pub block_id: Option<usize>,
    pub inst_id: Option<usize>,
}

impl fmt::Display for SirPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function {}", self.function_id)?;
        if let Some(block_id) = self.block_id {
            write!(f, ", block {}", block_id)?;
            if let Some(inst_id) = self.inst_id {
                write!(f, ", inst {}", inst_id)?;
            }
        }
        Ok(())
    }
}

impl ProgramUnit {
    pub fn validate_insts(&self) -> Result<(), SirValidationError> {
        for (function_id, function) in self.functions.iter().enumerate() {
            function.validate_insts(
                self,
                SirPosition {
                    function_id,
                    block_id: None,
                    inst_id: None,
                },
            )?;
        }
        Ok(())
    }
}

impl Function {
    pub fn validate_insts(
        &self,
        program_unit: &ProgramUnit,
        pos: SirPosition,
    ) -> Result<(), SirValidationError> {
        if self.num_args > self.num_vars {
            return Err(SirValidationError::ExcessNumArgs { pos });
        }
        for (block_id, block) in self.body.iter().enumerate() {
            block.validate_insts(
                program_unit,
                self,
                SirPosition {
                    block_id: Some(block_id),
                    ..pos
                },
            )?;
        }
        Ok(())
    }
}

impl BasicBlock {
    pub fn validate_insts(
        &self,
        program_unit: &ProgramUnit,
        function: &Function,
        pos: SirPosition,
    ) -> Result<(), SirValidationError> {
        for (inst_id, inst) in self.insts.iter().enumerate() {
            let is_last = inst_id == self.insts.len() - 1;
            if is_last && !inst.kind.is_tail() {
                return Err(SirValidationError::ExpectedTailInstruction { pos });
            } else if !is_last && inst.kind.is_tail() {
                return Err(SirValidationError::UnexpectedTailInstruction { pos });
            }
            inst.validate_inst(
                program_unit,
                function,
                SirPosition {
                    inst_id: Some(inst_id),
                    ..pos
                },
            )?;
        }
        Ok(())
    }
}

impl Inst {
    pub fn validate_inst(
        &self,
        program_unit: &ProgramUnit,
        function: &Function,
        pos: SirPosition,
    ) -> Result<(), SirValidationError> {
        match &self.kind {
            crate::sir::InstKind::Jump { target } => {
                if *target >= function.body.len() {
                    return Err(SirValidationError::InvalidTargetBlock { pos });
                }
            }
            crate::sir::InstKind::Branch {
                cond,
                branch_then,
                branch_else,
            } => {
                if *cond >= function.num_vars {
                    return Err(SirValidationError::InvalidVariableId { pos });
                }
                if *branch_then >= function.body.len() || *branch_else >= function.body.len() {
                    return Err(SirValidationError::InvalidTargetBlock { pos });
                }
            }
            crate::sir::InstKind::Return { rhs } => {
                if *rhs >= function.num_vars {
                    return Err(SirValidationError::InvalidVariableId { pos });
                }
            }
            crate::sir::InstKind::Copy { lhs, rhs } => {
                if *lhs >= function.num_vars || *rhs >= function.num_vars {
                    return Err(SirValidationError::InvalidVariableId { pos });
                }
            }
            crate::sir::InstKind::Drop { rhs } => {
                if *rhs >= function.num_vars {
                    return Err(SirValidationError::InvalidVariableId { pos });
                }
            }
            crate::sir::InstKind::Literal { lhs, value: _ } => {
                if *lhs >= function.num_vars {
                    return Err(SirValidationError::InvalidVariableId { pos });
                }
            }
            crate::sir::InstKind::Closure { lhs, function_id } => {
                if *lhs >= function.num_vars {
                    return Err(SirValidationError::InvalidVariableId { pos });
                }
                if *function_id >= program_unit.functions.len() {
                    return Err(SirValidationError::InvalidFunctionId { pos });
                }
            }
            crate::sir::InstKind::Builtin { lhs, builtin: _ } => {
                if *lhs >= function.num_vars {
                    return Err(SirValidationError::InvalidVariableId { pos });
                }
            }
            crate::sir::InstKind::PushArg { value_ref } => {
                if *value_ref >= function.num_vars {
                    return Err(SirValidationError::InvalidVariableId { pos });
                }
            }
            crate::sir::InstKind::Call { lhs, callee } => {
                if *lhs >= function.num_vars || *callee >= function.num_vars {
                    return Err(SirValidationError::InvalidVariableId { pos });
                }
            }
        }
        Ok(())
    }
}
