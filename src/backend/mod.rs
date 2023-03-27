mod codegen;

use self::codegen::codegen;
use crate::analysis::Ctxt;
use crate::ast::visitor;
use crate::ast::{self, LetStmt};

pub fn compile(ctx: &Ctxt, krate: &ast::Crate) -> Result<(), ()> {
    let mut bctx = BackendCtxt::new(ctx);

    let analyzer = &mut StackAnalyzer { bctx: &mut bctx };
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
    fn visit_let_stmt(&mut self, let_stmt: &'ctx ast::LetStmt) {
        let LetStmt { ident, ty: _ty } = &let_stmt;
        self.bctx.locals.push(&ident.symbol);
    }
}
