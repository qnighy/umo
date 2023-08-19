use std::borrow::Cow;

use crate::cctx::CCtx;
use crate::sir::{BasicBlock, BuiltinKind, Function, InstKind, Literal, ProgramUnit};

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
    fn expand_shallow<'a>(&self, ty: &'a Type) -> Cow<'a, Type> {
        if let Type::Var { var_id: id } = ty {
            if let Some(ty) = &self.ty_vars[*id] {
                return Cow::Owned(self.expand_shallow(ty).into_owned());
            }
        }
        Cow::Borrowed(ty)
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
            (Type::Unit, Type::Unit) => Ok(()),
            (Type::String, Type::String) => Ok(()),
            (Type::Integer, Type::Integer) => Ok(()),
            (Type::Bool, Type::Bool) => Ok(()),
            (
                Type::Function {
                    args: args1,
                    ret: ret1,
                },
                Type::Function {
                    args: args2,
                    ret: ret2,
                },
            ) => {
                if args1.len() != args2.len() {
                    return Err(TypeError);
                }
                for (arg1, arg2) in args1.iter().zip(args2) {
                    self.unify(arg1, arg2)?;
                }
                self.unify(ret1, ret2)
            }
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
            Type::Unit => false,
            Type::String => false,
            Type::Integer => false,
            Type::Bool => false,
            Type::Function { args, ret } => {
                args.iter().any(|arg| self.has_ty_var(arg, needle_id))
                    || self.has_ty_var(ret, needle_id)
            }
        }
    }
    #[allow(unused)] // TODO: remove it later
    fn has_any_ty_var(&self, ty: &Type) -> bool {
        match ty {
            Type::Var { var_id } => {
                if let Some(ty) = &self.ty_vars[*var_id] {
                    self.has_any_ty_var(ty)
                } else {
                    true
                }
            }
            Type::Unit => false,
            Type::String => false,
            Type::Integer => false,
            Type::Bool => false,
            Type::Function { args, ret } => {
                args.iter().any(|arg| self.has_any_ty_var(arg)) || self.has_any_ty_var(ret)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PTyCtx {
    functions: Vec<FunctionType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FunctionType {
    args: Vec<Type>,
    ret: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    vars: Vec<Type>,
}

pub fn typecheck(cctx: &CCtx, program_unit: &ProgramUnit) -> Result<(), TypeError> {
    let mut ty_ctx = TyCtx { ty_vars: vec![] };
    let pctx = PTyCtx {
        functions: program_unit
            .functions
            .iter()
            .map(|function| FunctionType {
                args: (0..function.num_args).map(|_| ty_ctx.fresh()).collect(),
                ret: ty_ctx.fresh(),
            })
            .collect(),
    };
    for (function, function_type) in program_unit.functions.iter().zip(&pctx.functions) {
        typecheck_function(cctx, &mut ty_ctx, &pctx, function, function_type)?;
    }
    Ok(())
}

fn typecheck_function(
    cctx: &CCtx,
    ty_ctx: &mut TyCtx,
    pctx: &PTyCtx,
    function: &Function,
    function_type: &FunctionType,
) -> Result<(), TypeError> {
    let mut state = State {
        vars: (0..function.num_vars).map(|_| ty_ctx.fresh()).collect(),
    };
    for (arg_var_type, arg_type) in state.vars.iter().zip(&function_type.args) {
        ty_ctx.unify(arg_var_type, arg_type)?;
    }
    for bb in &function.body {
        typecheck_bb(
            cctx,
            ty_ctx,
            pctx,
            &mut state,
            function,
            bb,
            &function_type.ret,
        )?;
    }
    // for ty in &state.vars {
    //     if ty_ctx.has_any_ty_var(ty) {
    //         return Err(TypeError);
    //     }
    // }
    // TODO: also check liveness
    Ok(())
}
fn typecheck_bb(
    cctx: &CCtx,
    ty_ctx: &mut TyCtx,
    pctx: &PTyCtx,
    state: &mut State,
    function: &Function,
    bb: &BasicBlock,
    return_type: &Type,
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
            InstKind::Return { rhs } => {
                ty_ctx.unify(&state.vars[*rhs], return_type)?;
            }
            InstKind::Copy { lhs, rhs } => {
                ty_ctx.unify(&state.vars[*lhs], &state.vars[*rhs])?;
            }
            InstKind::Drop { .. } => {}
            InstKind::Literal { lhs, value } => {
                ty_ctx.unify(&state.vars[*lhs], &Type::of_literal(value))?;
            }
            InstKind::Closure { lhs, function_id } => {
                if !args.is_empty() {
                    todo!("Variable-capturing closure");
                }
                let function_type = &pctx.functions[*function_id];
                ty_ctx.unify(
                    &state.vars[*lhs],
                    &Type::Function {
                        args: function_type.args.clone(),
                        ret: Box::new(function_type.ret.clone()),
                    },
                )?;
            }
            InstKind::Builtin { lhs, builtin } => {
                ty_ctx.unify(&state.vars[*lhs], &builtin_type(*builtin))?;
            }
            InstKind::PushArg { value_ref } => {
                args.push(state.vars[*value_ref].clone());
            }
            InstKind::Call_ { lhs, callee } => {
                let callee_type = &state.vars[*callee];
                let (callee_args, callee_ret) =
                    match ty_ctx.expand_shallow(callee_type).into_owned() {
                        Type::Function { args, ret } => (args, ret),
                        _ => return Err(TypeError),
                    };
                if args.len() != callee_args.len() {
                    return Err(TypeError);
                }
                for (arg, callee_arg) in args.iter().zip(callee_args) {
                    ty_ctx.unify(arg, &callee_arg)?;
                }
                ty_ctx.unify(&state.vars[*lhs], &callee_ret)?;
                args.clear();
            }
        }
    }
    if !args.is_empty() {
        return Err(TypeError);
    }
    Ok(())
}

fn builtin_type(f: BuiltinKind) -> Type {
    match f {
        BuiltinKind::Add => Type::Function {
            args: vec![Type::Integer, Type::Integer],
            ret: Box::new(Type::Integer),
        },
        BuiltinKind::Lt => Type::Function {
            args: vec![Type::Integer, Type::Integer],
            ret: Box::new(Type::Bool),
        },
        BuiltinKind::Puts => Type::Function {
            args: vec![Type::String],
            ret: Box::new(Type::Unit),
        },
        BuiltinKind::Puti => Type::Function {
            args: vec![Type::Integer],
            ret: Box::new(Type::Unit),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Type {
    Unit,
    String,
    Integer,
    Bool,
    Function { args: Vec<Type>, ret: Box<Type> },
    Var { var_id: usize },
}

impl Type {
    fn of_literal(literal: &Literal) -> Self {
        match literal {
            Literal::Unit => Self::Unit,
            Literal::Integer(_) => Self::Integer,
            Literal::Bool(_) => Self::Bool,
            Literal::String(_) => Self::String,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sir::testing::{insts, FunctionTestingExt, ProgramUnitTestingExt};
    use crate::sir::Function;

    #[test]
    fn test_typecheck_success() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::describe(
            0,
            |desc, (x, tmp1, puti1, tmp2), (entry,)| {
                desc.block(
                    entry,
                    vec![
                        insts::builtin(puti1, BuiltinKind::Puti),
                        insts::integer_literal(x, 42),
                        insts::push_arg(x),
                        insts::call(tmp2, puti1),
                        insts::unit_literal(tmp1),
                        insts::return_(tmp1),
                    ],
                );
            },
        ));
        assert!(typecheck(&cctx, &program_unit).is_ok());
    }

    #[test]
    fn test_typecheck_failure_too_few_arg() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::simple(0, |(tmp1, puti1, tmp2)| {
            vec![
                insts::builtin(puti1, BuiltinKind::Puti),
                insts::call(tmp2, puti1),
                insts::unit_literal(tmp1),
                insts::return_(tmp1),
            ]
        }));
        assert!(typecheck(&cctx, &program_unit).is_err());
    }

    #[test]
    fn test_typecheck_failure_too_many_arg() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::simple(0, |(x, tmp1, puti1, tmp2)| {
            vec![
                insts::integer_literal(x, 42),
                insts::builtin(puti1, BuiltinKind::Puti),
                insts::push_arg(x),
                insts::push_arg(x),
                insts::call(tmp2, puti1),
                insts::unit_literal(tmp1),
                insts::return_(tmp1),
            ]
        }));
        assert!(typecheck(&cctx, &program_unit).is_err());
    }

    #[test]
    fn test_typecheck_failure_arg_type_mismatch() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::simple(0, |(x, tmp1, puti1, tmp2)| {
            vec![
                insts::string_literal(x, "Hello, world!"),
                insts::builtin(puti1, BuiltinKind::Puti),
                insts::push_arg(x),
                insts::call(tmp2, puti1),
                insts::unit_literal(tmp1),
                insts::return_(tmp1),
            ]
        }));
        assert!(typecheck(&cctx, &program_unit).is_err());
    }

    #[test]
    fn test_typecheck_failure_runaway_arg() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::simple(0, |(x, tmp1)| {
            vec![
                insts::string_literal(x, "Hello, world!"),
                insts::unit_literal(tmp1),
                insts::push_arg(x),
                insts::return_(tmp1),
            ]
        }));
        assert!(typecheck(&cctx, &program_unit).is_err());
    }
}
