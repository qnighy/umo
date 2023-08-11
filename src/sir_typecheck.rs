use crate::cctx::CCtx;
use crate::sir::{BasicBlock, BuiltinKind, Function, InstKind, Literal};

#[derive(Debug)]
pub struct TypeError;

#[derive(Debug)]
struct TyCtx {
    ty_vars: Vec<Option<Type>>,
}

impl TyCtx {
    fn fresh(&mut self) -> Type {
        let ty = Type::Var {
            var_id: self.ty_vars.len(),
        };
        self.ty_vars.push(None);
        ty
    }
    fn unify(&mut self, ty1: &Type, ty2: &Type) -> Result<(), TypeError> {
        if let Type::Var { var_id: id } = ty1 {
            if let Some(ty1a) = &self.ty_vars[*id] {
                let ty1a = ty1a.clone();
                return self.unify(&ty1a, ty2);
            }
        }
        if let Type::Var { var_id: id } = ty2 {
            if let Some(ty2a) = &self.ty_vars[*id] {
                let ty2a = ty2a.clone();
                return self.unify(ty1, &ty2a);
            }
        }
        match (ty1, ty2) {
            (Type::Var { var_id: id1 }, Type::Var { var_id: id2 }) if id1 == id2 => Ok(()),
            (Type::Var { var_id: id1 }, ty2) => {
                if self.has_ty_var(ty2, *id1) {
                    return Err(TypeError);
                }
                self.ty_vars[*id1] = Some(ty2.clone());
                Ok(())
            }
            (ty1, Type::Var { var_id: id2 }) => {
                if self.has_ty_var(ty1, *id2) {
                    return Err(TypeError);
                }
                self.ty_vars[*id2] = Some(ty1.clone());
                Ok(())
            }
            (Type::String, Type::String) => Ok(()),
            (Type::Integer, Type::Integer) => Ok(()),
            (Type::Bool, Type::Bool) => Ok(()),
            _ => Err(TypeError),
        }
    }
    fn has_ty_var(&self, ty: &Type, needle_id: usize) -> bool {
        match ty {
            Type::Var { var_id: id } => {
                if *id == needle_id {
                    true
                } else if let Some(ty) = &self.ty_vars[*id] {
                    self.has_ty_var(ty, needle_id)
                } else {
                    false
                }
            }
            Type::String => false,
            Type::Integer => false,
            Type::Bool => false,
        }
    }
    fn has_any_ty_var(&self, ty: &Type) -> bool {
        match ty {
            Type::Var { var_id } => {
                if let Some(ty) = &self.ty_vars[*var_id] {
                    self.has_any_ty_var(ty)
                } else {
                    true
                }
            }
            Type::String => false,
            Type::Integer => false,
            Type::Bool => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    vars: Vec<Type>,
}

pub fn typecheck(cctx: &CCtx, function: &Function) -> Result<(), TypeError> {
    let mut ty_ctx = TyCtx { ty_vars: vec![] };
    let mut state = State {
        vars: (0..function.num_vars).map(|_| ty_ctx.fresh()).collect(),
    };
    for bb in &function.body {
        typecheck_bb(cctx, &mut ty_ctx, &mut state, function, bb)?;
    }
    for ty in &state.vars {
        if ty_ctx.has_any_ty_var(ty) {
            return Err(TypeError);
        }
    }
    // TODO: also check liveness
    Ok(())
}
fn typecheck_bb(
    cctx: &CCtx,
    ty_ctx: &mut TyCtx,
    state: &mut State,
    function: &Function,
    bb: &BasicBlock,
) -> Result<(), TypeError> {
    let mut args = vec![];
    for inst in &bb.insts {
        match &inst.kind {
            InstKind::Jump { target } => {
                if *target >= function.body.len() {
                    return Err(TypeError);
                }
            }
            InstKind::Branch {
                cond,
                branch_then,
                branch_else,
            } => {
                if *branch_then >= function.body.len() {
                    return Err(TypeError);
                }
                if *branch_else >= function.body.len() {
                    return Err(TypeError);
                }
                ty_ctx.unify(&state.vars[*cond], &Type::Bool)?;
            }
            InstKind::Return => {}
            InstKind::Copy { lhs, rhs } => {
                ty_ctx.unify(&state.vars[*lhs], &state.vars[*rhs])?;
            }
            InstKind::Drop { .. } => {}
            InstKind::Literal { lhs, value } => {
                ty_ctx.unify(&state.vars[*lhs], &Type::of_literal(value))?;
            }
            InstKind::PushArg { value_ref } => {
                args.push(state.vars[*value_ref].clone());
            }
            InstKind::CallBuiltin { lhs, builtin: f } => {
                let args = std::mem::replace(&mut args, vec![]);
                let return_type = typecheck_builtin(cctx, ty_ctx, *f, args)?;
                if let Some(lhs) = lhs {
                    ty_ctx.unify(&state.vars[*lhs], &return_type)?;
                }
            }
        }
    }
    if !args.is_empty() {
        return Err(TypeError);
    }
    Ok(())
}

fn typecheck_builtin(
    _cctx: &CCtx,
    ty_ctx: &mut TyCtx,
    f: BuiltinKind,
    args: Vec<Type>,
) -> Result<Type, TypeError> {
    match f {
        BuiltinKind::Add => {
            if args.len() != 2 {
                return Err(TypeError);
            }
            ty_ctx.unify(&args[0], &Type::Integer)?;
            ty_ctx.unify(&args[1], &Type::Integer)?;
            Ok(Type::Integer)
        }
        BuiltinKind::Lt => {
            if args.len() != 2 {
                return Err(TypeError);
            }
            ty_ctx.unify(&args[0], &Type::Integer)?;
            ty_ctx.unify(&args[1], &Type::Integer)?;
            Ok(Type::Bool)
        }
        BuiltinKind::Puts => {
            if args.len() != 1 {
                return Err(TypeError);
            }
            ty_ctx.unify(&args[0], &Type::String)?;
            Ok(Type::Integer)
        }
        BuiltinKind::Puti => {
            if args.len() != 1 {
                return Err(TypeError);
            }
            ty_ctx.unify(&args[0], &Type::Integer)?;
            Ok(Type::Integer)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Type {
    String,
    Integer,
    Bool,
    Var { var_id: usize },
}

impl Type {
    fn of_literal(literal: &Literal) -> Self {
        match literal {
            Literal::Integer(_) => Self::Integer,
            Literal::Bool(_) => Self::Bool,
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
                insts::return_(),
            ])]
        });
        assert!(typecheck(&cctx, &bb).is_ok());
    }

    #[test]
    fn test_typecheck_failure_too_few_arg() {
        let cctx = CCtx::new();
        let bb =
            Function::describe(|()| vec![BasicBlock::new(vec![insts::puti(), insts::return_()])]);
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
                insts::return_(),
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
                insts::return_(),
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
                insts::return_(),
            ])]
        });
        assert!(typecheck(&cctx, &bb).is_err());
    }
}
