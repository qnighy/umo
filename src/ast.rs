#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    // Var { name: String },
    #[allow(unused)] // TODO: remove this annotation later
    // TODO: use BigInt
    IntegerLiteral { value: i32 },
    #[allow(unused)] // TODO: remove this annotation later
    Add { lhs: Box<Expr>, rhs: Box<Expr> },
}

#[cfg(test)]
pub mod testing {
    pub mod exprs {
        use super::super::*;

        // pub fn var(name: &str) -> Expr {
        //     Expr::Var {
        //         name: name.to_string(),
        //     }
        // }

        pub fn integer_literal(value: i32) -> Expr {
            Expr::IntegerLiteral { value }
        }

        pub fn add(lhs: Expr, rhs: Expr) -> Expr {
            Expr::Add {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
    }
}
