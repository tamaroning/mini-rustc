use crate::ast::visitor::{self};
use crate::ast::{self};
use crate::middle::Ctxt;
use crate::resolve::NameBinding;
use std::collections::HashMap;

const LOCAL_OR_PARAM_START_OFFSET: u32 = 8;

/// Struct representing a single stack frame
/// FIXME: shadowing, scope. See Ctxt
#[derive(Debug)]
pub struct FrameInfo {
    pub size: u32,
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

/// Struct representing a local variable including arguments
#[derive(Debug)]
pub struct LocalInfo {
    pub offset: u32,
    pub size: u32,
    // align: u32,
}

impl FrameInfo {
    /// Collect all locals (including args) and create `FrameInfo`
    pub fn compute(ctx: &Ctxt, func: &ast::Func) -> Self {
        let mut analyzer = FuncAnalyzer {
            ctx,
            current_offset: LOCAL_OR_PARAM_START_OFFSET,
            frame_info: FrameInfo::new(),
        };
        visitor::go_func(&mut analyzer, func);

        analyzer.frame_info
    }
}

struct FuncAnalyzer<'ctx> {
    ctx: &'ctx Ctxt,
    current_offset: u32,
    frame_info: FrameInfo,
    // FIXME: alignment
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
            let param_size = self.ctx.get_size(param_ty);
            self.current_offset += param_size;
            self.frame_info.size += param_size;
            let local = LocalInfo {
                offset: self.current_offset,
                size: param_size,
            };
            let name_binding = self.ctx.resolver.resolve_ident(param).unwrap();
            self.frame_info.locals.insert(name_binding, local);
        }
    }

    fn visit_let_stmt(&mut self, let_stmt: &'ctx ast::LetStmt) {
        let size = self.ctx.get_size(&let_stmt.ty);
        self.current_offset += size;
        self.frame_info.size += size;
        let local = LocalInfo {
            offset: self.current_offset,
            size,
        };
        let name_binding = self.ctx.resolver.resolve_ident(&let_stmt.ident).unwrap();
        self.frame_info.locals.insert(name_binding, local);
    }
}
