use std::fmt;

use thiserror::Error;

use option_cell::OptionCell;

#[derive(Debug, Error)]
#[error("Unification failure")]
pub struct UnificationFailure;

#[derive(Debug, Default)]
pub struct TyCtx {
    vars: Vec<Option<Type>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    MetaVar { var_id: usize },
    Unit,
    String,
    Integer,
    Bool,
    Function { args: Vec<Type>, ret: Box<Type> },
}

impl Type {
    pub fn fresh(ctx: &mut TyCtx) -> Self {
        let var_id = ctx.vars.len();
        ctx.vars.push(None);
        Type::MetaVar { var_id }
    }
    pub fn unit() -> Self {
        Type::Unit
    }
    pub fn string() -> Self {
        Type::String
    }
    pub fn integer() -> Self {
        Type::Integer
    }
    pub fn bool() -> Self {
        Type::Bool
    }
    pub fn function(args: Vec<Type>, ret: Type) -> Self {
        Type::Function {
            args,
            ret: Box::new(ret),
        }
    }

    pub fn view<'a>(&'a self, ctx: &'a TyCtx) -> TypeView<'a> {
        TypeView { type_: self, ctx }
    }

    pub fn resolve<'a>(&'a self, ctx: &'a TyCtx) -> &'a Type {
        let mut ty = self;
        loop {
            match ty {
                Type::MetaVar { var_id } => {
                    if let Some(next_ty) = &ctx.vars[*var_id] {
                        ty = next_ty;
                        continue;
                    }
                }
                _ => {}
            }
            return ty;
        }
    }
    fn resolve2<'a>(&'a self, vars: &'a [OptionCell<Type>]) -> &'a Type {
        let mut ty = self;
        loop {
            match ty {
                Type::MetaVar { var_id } => {
                    if let Some(next_ty) = vars[*var_id].get() {
                        ty = next_ty;
                        continue;
                    }
                }
                _ => {}
            }
            return ty;
        }
    }

    pub fn unify(&self, other: &Self, ctx: &mut TyCtx) -> Result<(), UnificationFailure> {
        let vars = OptionCell::from_mut_slice(&mut ctx.vars);
        self.unify_impl(other, vars)
    }
    fn unify_impl(
        &self,
        other: &Self,
        vars: &[OptionCell<Type>],
    ) -> Result<(), UnificationFailure> {
        let ty1 = self.resolve2(vars);
        let ty2 = other.resolve2(vars);
        match (ty1, ty2) {
            (Type::MetaVar { var_id: var_id1 }, Type::MetaVar { var_id: var_id2 })
                if var_id1 == var_id2 =>
            {
                Ok(())
            }
            (Type::MetaVar { var_id }, _) => {
                if ty2.has_fv(*var_id, vars) {
                    return Err(UnificationFailure);
                }
                vars[*var_id].set(ty2.clone()).unwrap();
                Ok(())
            }
            (_, Type::MetaVar { var_id }) => {
                if ty1.has_fv(*var_id, vars) {
                    return Err(UnificationFailure);
                }
                vars[*var_id].set(ty1.clone()).unwrap();
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
                    return Err(UnificationFailure);
                }
                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    arg1.unify_impl(arg2, vars)?;
                }
                ret1.unify_impl(ret2, vars)?;
                Ok(())
            }
            _ => Err(UnificationFailure),
        }
    }

    fn has_fv(&self, var_id: usize, vars: &[OptionCell<Type>]) -> bool {
        let ty = self.resolve2(vars);
        match ty {
            Type::MetaVar { var_id: id } => *id == var_id,
            Type::Unit => false,
            Type::String => false,
            Type::Integer => false,
            Type::Bool => false,
            Type::Function { args, ret } => {
                args.iter().any(|ty| ty.has_fv(var_id, vars)) || ret.has_fv(var_id, vars)
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct TypeView<'a> {
    type_: &'a Type,
    ctx: &'a TyCtx,
}

impl PartialEq for TypeView<'_> {
    fn eq(&self, other: &Self) -> bool {
        let ty1 = self.type_.resolve(self.ctx);
        let ty2 = other.type_.resolve(other.ctx);
        match (ty1, ty2) {
            (Type::MetaVar { var_id: id1 }, Type::MetaVar { var_id: id2 }) => id1 == id2,
            (Type::Unit, Type::Unit) => true,
            (Type::String, Type::String) => true,
            (Type::Integer, Type::Integer) => true,
            (Type::Bool, Type::Bool) => true,
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
                args1.len() == args2.len()
                    && args1.iter().zip(args2.iter()).all(|(ty1, ty2)| {
                        TypeView {
                            type_: ty1,
                            ctx: self.ctx,
                        } == TypeView {
                            type_: ty2,
                            ctx: other.ctx,
                        }
                    })
                    && TypeView {
                        type_: ret1,
                        ctx: self.ctx,
                    } == TypeView {
                        type_: ret2,
                        ctx: other.ctx,
                    }
            }
            _ => false,
        }
    }
}

impl Eq for TypeView<'_> {}

impl fmt::Debug for TypeView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ty = self.type_.resolve(self.ctx);
        match ty {
            Type::MetaVar { var_id } => write!(f, "var_{}", var_id),
            Type::Unit => write!(f, "Type::unit()"),
            Type::String => write!(f, "Type::string()"),
            Type::Integer => write!(f, "Type::integer()"),
            Type::Bool => write!(f, "Type::bool()"),
            Type::Function { args, ret } => f
                .debug_tuple("Type::function")
                .field(
                    &args
                        .iter()
                        .map(|ty| TypeView {
                            type_: ty,
                            ctx: self.ctx,
                        })
                        .collect::<Vec<_>>(),
                )
                .field(&TypeView {
                    type_: ret,
                    ctx: self.ctx,
                })
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_simple() {
        let mut ctx = TyCtx::default();
        assert!(Type::unit().unify(&Type::unit(), &mut ctx).is_ok());
        assert!(Type::string().unify(&Type::string(), &mut ctx).is_ok());
        assert!(Type::integer().unify(&Type::integer(), &mut ctx).is_ok());
        assert!(Type::bool().unify(&Type::bool(), &mut ctx).is_ok());
        assert!(
            Type::function(vec![Type::unit(), Type::string()], Type::integer())
                .unify(
                    &Type::function(vec![Type::unit(), Type::string()], Type::integer()),
                    &mut ctx
                )
                .is_ok()
        );

        assert!(Type::unit().unify(&Type::string(), &mut ctx).is_err());
        assert!(Type::string().unify(&Type::integer(), &mut ctx).is_err());
        assert!(Type::integer().unify(&Type::bool(), &mut ctx).is_err());
        assert!(Type::bool()
            .unify(&Type::function(vec![], Type::unit()), &mut ctx)
            .is_err());
    }

    #[test]
    fn test_unify_var_var() {
        let mut ctx = TyCtx::default();

        let ty1 = Type::fresh(&mut ctx);
        let ty2 = Type::fresh(&mut ctx);
        assert_ne!(ty1.view(&ctx), ty2.view(&ctx));
        assert!(ty1.unify(&ty2, &mut ctx).is_ok());
        assert_eq!(ty1.view(&ctx), ty2.view(&ctx));
    }

    #[test]
    fn test_unify_var_self() {
        let mut ctx = TyCtx::default();

        let ty1 = Type::fresh(&mut ctx);
        assert!(ty1.unify(&ty1, &mut ctx).is_ok());
    }

    #[test]
    fn test_unify_var_concrete() {
        let mut ctx = TyCtx::default();

        {
            let ty1 = Type::fresh(&mut ctx);
            let ty2 = Type::integer();
            assert_ne!(ty1.view(&ctx), ty2.view(&ctx));
            assert!(ty1.unify(&ty2, &mut ctx).is_ok());
            assert_eq!(ty1.view(&ctx), ty2.view(&ctx));
        }

        {
            let ty1 = Type::integer();
            let ty2 = Type::fresh(&mut ctx);
            assert_ne!(ty1.view(&ctx), ty2.view(&ctx));
            assert!(ty1.unify(&ty2, &mut ctx).is_ok());
            assert_eq!(ty1.view(&ctx), ty2.view(&ctx));
        }
    }

    #[test]
    fn test_unify_concrete_unified() {
        let mut ctx = TyCtx::default();

        {
            let ty1 = Type::fresh(&mut ctx);
            let ty2 = Type::integer();
            let ty3 = Type::unit();
            ty1.unify(&ty2, &mut ctx).unwrap();
            assert!(ty1.unify(&ty3, &mut ctx).is_err());
        }

        {
            let ty1 = Type::fresh(&mut ctx);
            let ty2 = Type::integer();
            let ty3 = Type::unit();
            ty1.unify(&ty2, &mut ctx).unwrap();
            assert!(ty3.unify(&ty1, &mut ctx).is_err());
        }
    }

    #[test]
    fn test_unify_arg() {
        let mut ctx = TyCtx::default();

        {
            let var1 = Type::fresh(&mut ctx);
            let ty1 = Type::function(vec![var1.clone()], Type::unit());
            let ty2 = Type::function(vec![Type::integer()], Type::unit());
            assert!(ty1.unify(&ty2, &mut ctx).is_ok());
            assert_eq!(var1.view(&ctx), Type::integer().view(&ctx));
        }
    }
}
