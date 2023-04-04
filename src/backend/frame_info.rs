use crate::ast::visitor::{self};
use crate::ast::{self, Ident};
use crate::middle::ty::Ty;
use crate::middle::Ctxt;
use crate::resolve::NameBinding;
use std::collections::HashMap;
use std::rc::Rc;

const LOCAL_OR_PARAM_START_OFFSET: usize = 0;
/// RSP (and RBP??) must align by 16 bytes
const FRAME_SIZE_ALIGN: usize = 16;

/// Struct representing a single stack frame
/// FIXME: shadowing, scope. See Ctxt
#[derive(Debug)]
pub struct FrameInfo {
    // TODO: frame size should align by 16 bytes because rsp must do so.
    pub size: usize,
    // local variables and parameters to LocalInfo mappings
    pub locals: HashMap<NameBinding, LocalInfo>,
}

impl FrameInfo {
    fn new() -> Self {
        FrameInfo {
            size: LOCAL_OR_PARAM_START_OFFSET,
            locals: HashMap::new(),
        }
    }
}

impl FrameInfo {
    /// Collect all locals (including args) and create `FrameInfo`
    pub fn compute(ctx: &mut Ctxt, func: &ast::Func) -> Self {
        let mut analyzer = FuncAnalyzer {
            ctx,
            current_offset: LOCAL_OR_PARAM_START_OFFSET,
            frame_info: FrameInfo::new(),
        };
        visitor::go_func(&mut analyzer, func);

        analyzer.frame_info
    }

    fn add_padding(&mut self, padd_size: usize) {
        self.size += padd_size;
    }

    fn add_local(&mut self, size: usize) {
        self.size += size;
    }
}

/// Struct representing a local variable on a stack
#[derive(Debug)]
pub struct LocalInfo {
    pub offset: usize,
    pub size: usize,
}

struct FuncAnalyzer<'ctx> {
    ctx: &'ctx mut Ctxt,
    current_offset: usize,
    frame_info: FrameInfo,
}

impl FuncAnalyzer<'_> {
    fn add_local(&mut self, ident: &Ident, ty: &Rc<Ty>) {
        let size = self.ctx.get_size(ty);
        self.current_offset += size;

        // insert padding
        let align = self.ctx.get_align(ty);
        assert!(align != 0);
        let padding = if self.current_offset % align == 0 {
            0
        } else {
            align - self.current_offset % align
        };
        self.current_offset += padding;
        self.frame_info.add_padding(padding);

        // add LocalInfo to the FramInfo
        self.frame_info.add_local(size);
        let local = LocalInfo {
            offset: self.current_offset,
            size,
        };
        let name_binding = self.ctx.resolver.resolve_ident(ident).unwrap();
        self.frame_info.locals.insert(name_binding, local);
    }

    fn finalize(&mut self) {
        let align = FRAME_SIZE_ALIGN;
        let padding = if self.current_offset % align == 0 {
            0
        } else {
            align - self.current_offset % align
        };
        self.current_offset += padding;
        self.frame_info.add_padding(padding);
    }
}

impl<'ctx: 'a, 'a> ast::visitor::Visitor<'ctx> for FuncAnalyzer<'a> {
    // ↑ stack growth
    //   (low addr)
    // |      ...       |
    // +----------------+
    // |     data...    |
    // +----------------+
    // | ↓ data growth  |
    // +----------------+
    // |      ...       |
    //   stack bottom
    //   (high addr)
    fn visit_func(&mut self, func: &'ctx ast::Func) {
        for (param, param_ty) in &func.params {
            self.add_local(param, param_ty);
        }
    }

    fn visit_func_post(&mut self, _func: &'ctx ast::Func) {
        self.finalize();
    }

    fn visit_let_stmt(&mut self, let_stmt: &'ctx ast::LetStmt) {
        self.add_local(&let_stmt.ident, &let_stmt.ty);
    }
}
