mod codegen;

use self::codegen::codegen;
use crate::analysis::Ctxt;
use crate::ast::visitor;
use crate::ast::{self, visitor::Visitor, LetStmt};

pub fn compile(ctx: &Ctxt, krate: &ast::Crate) -> Result<(), ()> {
    let mut bctx = BackendCtxt::new(ctx);

    let analyzer: &mut dyn Visitor = &mut StackAnalyzer { bctx: &mut bctx };
    visitor::go(analyzer, krate);

    codegen(&bctx, krate)?;

    Ok(())
}

pub struct BackendCtxt<'a, 'ctx> {
    locals: Vec<&'ctx String>,
    ctx: &'a Ctxt,
}

impl<'a, 'ctx> BackendCtxt<'a, 'ctx> {
    fn new(ctx: &'a Ctxt) -> Self {
        BackendCtxt {
            locals: Vec::new(),
            ctx,
        }
    }
}

struct StackAnalyzer<'ctx, 'a, 'b> {
    bctx: &'a mut BackendCtxt<'b, 'ctx>,
}

impl<'ctx> ast::visitor::Visitor<'ctx> for StackAnalyzer<'ctx, '_, '_> {
    fn visit_crate(&mut self, _krate: &'ctx ast::Crate) {}

    fn visit_stmt(&mut self, _stmt: &'ctx ast::Stmt) {}

    fn visit_expr(&mut self, _expr: &'ctx ast::Expr) {}

    fn visit_let_stmt(&mut self, let_stmt: &'ctx ast::LetStmt) {
        let LetStmt { ident } = &let_stmt;
        self.bctx.locals.push(&ident.symbol);
    }

    fn visit_ident(&mut self, _ident: &'ctx ast::Ident) {}
}
