use crate::{
    ast::{self, Crate, ExprKind},
    middle::Ctxt,
};

pub fn analyze<'ctx>(ctx: &'ctx mut Ctxt, krate: &'ctx Crate) {
    let mut checker = Checker::new(ctx);
    ast::visitor::go(&mut checker, krate);
}

struct Checker<'chk> {
    ctx: &'chk mut Ctxt,
    is_let_initializer: bool,
}

impl<'ctx, 'chk: 'ctx> Checker<'ctx> {
    fn new(ctx: &'ctx mut Ctxt) -> Self {
        Checker {
            ctx,
            is_let_initializer: false,
        }
    }
}

// https://doc.rust-lang.org/reference/expressions.html?highlight=rvalue#place-expressions-and-value-expressions
impl<'ctx> ast::visitor::Visitor<'ctx> for Checker<'ctx> {
    fn visit_stmt(&mut self, _: &'ctx ast::LetStmt) {
        self.is_let_initializer = true;
    }

    fn visit_expr(&mut self, expr: &'ctx ast::Expr) {
        if self.is_let_initializer {
            self.ctx.register_lvalue(expr.id);
            self.is_let_initializer = false;
        }
    }

    fn visit_expr_post(&mut self, expr: &'ctx ast::Expr) {
        match &expr.kind {
            ExprKind::Index(operand, _) | ExprKind::Field(operand, _) => {
                self.ctx.register_lvalue(operand.id);
            }
            _ => (),
        }
    }
}
