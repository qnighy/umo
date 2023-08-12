use crate::ast::Expr;
use crate::sir;

#[allow(unused)] // TODO: remove this annotation later
pub fn lower(expr: &Expr) -> sir::Function {
    let num_args = 0;
    let mut function =
        sir::Function::new(num_args, num_args, vec![sir::BasicBlock { insts: vec![] }]);
    let mut bb_id = 0;
    let result_var = lower_expr(expr, &mut function, &mut bb_id);
    function.body[bb_id]
        .insts
        .push(sir::Inst::new(sir::InstKind::Return { rhs: result_var }));
    function
}

fn lower_expr(expr: &Expr, function: &mut sir::Function, bb_id: &mut usize) -> usize {
    let result_var = function.num_vars;
    function.num_vars += 1;
    match expr {
        Expr::IntegerLiteral { value } => {
            function.body[*bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Literal {
                    lhs: result_var,
                    value: sir::Literal::Integer(*value),
                }));
        }
        Expr::Add { lhs, rhs } => {
            let lhs_var = lower_expr(lhs, function, bb_id);
            let rhs_var = lower_expr(rhs, function, bb_id);

            let bb = &mut function.body[*bb_id];
            bb.insts.push(sir::Inst::new(sir::InstKind::PushArg {
                value_ref: lhs_var,
            }));
            bb.insts.push(sir::Inst::new(sir::InstKind::PushArg {
                value_ref: rhs_var,
            }));
            bb.insts.push(sir::Inst::new(sir::InstKind::CallBuiltin {
                lhs: Some(result_var),
                builtin: sir::BuiltinKind::Add,
            }));
        }
    }
    result_var
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::testing::exprs;
    use crate::sir::testing::{insts, FunctionTestingExt};

    #[test]
    fn test_lower() {
        let expr = exprs::add(exprs::integer_literal(1), exprs::integer_literal(2));
        let function = lower(&expr);
        assert_eq!(
            function,
            sir::Function::describe(0, |desc, (tmp1, tmp2, tmp3), (entry,)| {
                desc.block(
                    entry,
                    vec![
                        insts::integer_literal(tmp2, 1),
                        insts::integer_literal(tmp3, 2),
                        insts::push_arg(tmp2),
                        insts::push_arg(tmp3),
                        insts::add(tmp1),
                        insts::return_(tmp1),
                    ],
                );
            })
        );
    }
}
