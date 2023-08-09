use crate::cctx::CCtx;
use crate::sir::{BasicBlock, BuiltinKind, Function, InstKind, Literal};

#[derive(Debug)]
pub struct TypeError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    vars: Vec<Option<Type>>,
    args: Vec<Type>,
}

pub fn typecheck(cctx: &CCtx, function: &Function) -> Result<(), TypeError> {
    for bb in &function.body {
        typecheck_bb(cctx, function, bb)?;
    }
    Ok(())
}
fn typecheck_bb(cctx: &CCtx, function: &Function, bb: &BasicBlock) -> Result<(), TypeError> {
    let mut state = State {
        vars: vec![None; function.num_vars],
        args: vec![],
    };
    for inst in &bb.insts {
        match &inst.kind {
            InstKind::Copy { lhs, rhs } => {
                let rhs_type = state.vars[*rhs].as_ref().ok_or_else(|| TypeError)?.clone();
                state.vars[*lhs] = Some(rhs_type);
            }
            InstKind::Literal { lhs, value } => {
                let value_type = Type::of_literal(value);
                state.vars[*lhs] = Some(value_type);
            }
            InstKind::PushArg { value_ref } => {
                let value_type = state.vars[*value_ref]
                    .as_ref()
                    .ok_or_else(|| TypeError)?
                    .clone();
                state.args.push(value_type);
            }
            InstKind::CallBuiltin { lhs, builtin: f } => {
                let args = std::mem::replace(&mut state.args, vec![]);
                let return_type = typecheck_builtin(cctx, *f, args)?;
                if let Some(lhs) = lhs {
                    state.vars[*lhs] = Some(return_type);
                }
            }
        }
    }
    if !state.args.is_empty() {
        return Err(TypeError);
    }
    Ok(())
}

fn typecheck_builtin(_cctx: &CCtx, f: BuiltinKind, args: Vec<Type>) -> Result<Type, TypeError> {
    match f {
        BuiltinKind::Add => {
            if args.len() != 2 {
                return Err(TypeError);
            }
            if let Type::Integer = &args[0] {
            } else {
                return Err(TypeError);
            }
            if let Type::Integer = &args[1] {
            } else {
                return Err(TypeError);
            }
            Ok(Type::Integer)
        }
        BuiltinKind::Puts => {
            if args.len() != 1 {
                return Err(TypeError);
            }
            if let Type::String = &args[0] {
                Ok(Type::Integer)
            } else {
                Err(TypeError)
            }
        }
        BuiltinKind::Puti => {
            if args.len() != 1 {
                return Err(TypeError);
            }
            if let Type::Integer = &args[0] {
                Ok(Type::Integer)
            } else {
                Err(TypeError)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Type {
    String,
    Integer,
}

impl Type {
    fn of_literal(literal: &Literal) -> Self {
        match literal {
            Literal::Integer(_) => Self::Integer,
            Literal::String(_) => Self::String,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sir::testing::{insts, FunctionTestingExt};
    use crate::sir::{BasicBlock, Function};

    #[test]
    fn test_typecheck_success() {
        let cctx = CCtx::new();
        let bb = Function::describe(|(x,)| {
            vec![BasicBlock::new(vec![
                insts::integer_literal(x, 42),
                insts::push_arg(x),
                insts::puti(),
            ])]
        });
        assert!(typecheck(&cctx, &bb).is_ok());
    }

    #[test]
    fn test_typecheck_failure_too_few_arg() {
        let cctx = CCtx::new();
        let bb = Function::describe(|()| vec![BasicBlock::new(vec![insts::puti()])]);
        assert!(typecheck(&cctx, &bb).is_err());
    }

    #[test]
    fn test_typecheck_failure_too_many_arg() {
        let cctx = CCtx::new();
        let bb = Function::describe(|(x,)| {
            vec![BasicBlock::new(vec![
                insts::integer_literal(x, 42),
                insts::push_arg(x),
                insts::push_arg(x),
                insts::puti(),
            ])]
        });
        assert!(typecheck(&cctx, &bb).is_err());
    }

    #[test]
    fn test_typecheck_failure_arg_type_mismatch() {
        let cctx = CCtx::new();
        let bb = Function::describe(|(x,)| {
            vec![BasicBlock::new(vec![
                insts::string_literal(x, "Hello, world!"),
                insts::push_arg(x),
                insts::puti(),
            ])]
        });
        assert!(typecheck(&cctx, &bb).is_err());
    }

    #[test]
    fn test_typecheck_failure_runaway_arg() {
        let cctx = CCtx::new();
        let bb = Function::describe(|(x,)| {
            vec![BasicBlock::new(vec![
                insts::string_literal(x, "Hello, world!"),
                insts::push_arg(x),
            ])]
        });
        assert!(typecheck(&cctx, &bb).is_err());
    }
}
