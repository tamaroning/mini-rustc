use std::collections::HashMap;

use crate::ast;
use crate::ast::visitor::{self};
use crate::middle::Ctxt;

const INIT_LOCAL_OR_PARAM_OFFSET: u32 = 8;

// FIXME: shadowing, scope. See Ctxt
#[derive(Debug)]
pub struct FrameInfo<'a> {
    pub size: u32,
    pub locals: HashMap<&'a String, LocalInfo>,
    pub args: HashMap<&'a String, LocalInfo>,
}

impl FrameInfo<'_> {
    fn new() -> Self {
        FrameInfo {
            size: INIT_LOCAL_OR_PARAM_OFFSET,
            locals: HashMap::new(),
            args: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct LocalInfo {
    pub offset: u32,
    pub size: u32,
    // align: u32,
}

impl<'ctx> FrameInfo<'ctx> {
    pub fn compute(ctx: &'ctx Ctxt, func: &'ctx ast::Func) -> Self {
        let mut analyzer = FuncAnalyzer {
            ctx,
            current_offset: INIT_LOCAL_OR_PARAM_OFFSET,
            frame_info: FrameInfo::new(),
        };
        visitor::go_func(&mut analyzer, func);

        analyzer.frame_info
    }
}

struct FuncAnalyzer<'a> {
    ctx: &'a Ctxt,
    current_offset: u32,
    frame_info: FrameInfo<'a>,
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
        for (param_ident, param_ty) in &func.params {
            let param_size = self.ctx.get_size(&param_ty);
            self.current_offset += param_size;
            self.frame_info.size += param_size;
            let local = LocalInfo {
                offset: self.current_offset,
                size: param_size,
            };
            self.frame_info.args.insert(&param_ident.symbol, local);
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
        self.frame_info.locals.insert(&let_stmt.ident.symbol, local);
    }
}
