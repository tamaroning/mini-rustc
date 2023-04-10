use crate::ast::*;
use crate::hir;
use crate::hir::HirId;
use crate::hir::LocalDefId;
use crate::middle::Ctxt;

pub fn lower_crate(ctx: &mut Ctxt, krate: Crate) -> hir::Crate {
    let mut lower = ASTLower {
        ctx,
        next_hir_id: HirId::new(),
        next_def_id: LocalDefId::new(),
    };
    let translated = lower.lower_crate(krate);
    translated
}

struct ASTLower<'low> {
    ctx: &'low mut Ctxt,
    next_hir_id: HirId,
    next_def_id: LocalDefId,
}

impl ASTLower<'_> {
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

    pub fn lower_crate(&mut self, krate: Crate) -> hir::Crate {
        let mut items = vec![];
        for item in krate.items {
            items.push(self.lower_item(item));
        }
        hir::Crate {
            items,
            id: self.get_next_hir_id(),
        }
    }

    pub fn lower_item(&mut self, item: Item) -> hir::Item {
        todo!()
    }
}
