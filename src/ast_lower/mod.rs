use crate::ast::{self, *};
use crate::hir;
use crate::hir::HirId;
use crate::hir::LocalDefId;
use crate::middle::Ctxt;

pub fn lower_crate(ctx: &mut Ctxt, krate: &Crate) {
    ctx.resolve(&krate);

    let mut lower = ASTLower {
        ctx,
        next_hir_id: HirId::new(),
        next_def_id: LocalDefId::new(),

        current_modules: vec![],
    };
}

struct ASTLower<'low, 'ctx> {
    ctx: &'low mut Ctxt<'ctx>,
    next_hir_id: HirId,
    next_def_id: LocalDefId,

    current_modules: Vec<hir::Mod<'ctx>>,
}

impl<'low, 'ctx> ASTLower<'low, 'ctx> {
    fn get_next_hir_id(&mut self) -> HirId {
        let id = self.next_hir_id;
        self.next_hir_id = self.next_hir_id.next();
        id
    }

    fn get_next_def_id(&mut self) -> LocalDefId {
        let id = self.next_def_id;
        self.next_def_id = self.next_def_id.next();
        id
    }

    fn push_module(&mut self) {
        let hir_id = self.get_next_hir_id();
        self.current_modules.push(hir::Mod {
            items: vec![],
            id: hir_id,
        })
    }

    fn pop_module(&mut self) {
        let def_id = self.get_next_def_id();
        let module = self.current_modules.pop().unwrap();
        // TODO:
    }
}

impl<'low, 'ctx> ast::visitor::Visitor<'ctx> for ASTLower<'low, 'ctx> {
    fn visit_crate(&mut self, _krate: &'ctx Crate) {
        let defid = self.get_next_def_id();
        let hirid = self.get_next_hir_id();
    }

    fn visit_crate_post(&mut self, _krate: &'ctx Crate) {}
}
