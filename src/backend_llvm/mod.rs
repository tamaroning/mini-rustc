mod codegen_crate;
mod codegen_expr;
mod codegen_utils;
mod frame;
mod llvm;

use self::frame::Frame;
use self::llvm::*;
use crate::ast::Crate;
use crate::middle::ty::{AdtDef, Ty, TyKind};
use crate::middle::Ctxt;
use crate::resolve::CanonicalPath;
use std::collections::HashMap;
use std::rc::Rc;

pub fn compile<'ctx, 'gen: 'ctx>(ctx: &'gen mut Ctxt<'ctx>, krate: &'gen Crate) -> Result<(), ()> {
    let mut codegen = Codegen::new(ctx);
    codegen.go(krate)?;
    Ok(())
}

pub struct Codegen<'gen, 'ctx> {
    ctx: &'gen mut Ctxt<'ctx>,
    current_frame: Option<Frame>,
    ll_adt_defs: HashMap<Rc<CanonicalPath>, Rc<LLAdtDef>>,
    constants: Vec<Rc<LLConst>>,
    next_str_id: usize,
}

impl<'ctx, 'gen> Codegen<'ctx, 'gen> {
    fn new(ctx: &'gen mut Ctxt<'ctx>) -> Self {
        Codegen {
            ctx,
            current_frame: None,
            ll_adt_defs: HashMap::new(),
            constants: vec![],
            next_str_id: 1,
        }
    }

    pub fn get_fresh_str_name(&mut self) -> String {
        let i = self.next_str_id;
        self.next_str_id += 1;
        format!("@.str.{i}")
    }

    // TODO: memoize
    fn ty_to_llty(&self, ty: &Ty) -> LLTy {
        match &ty.kind {
            TyKind::Unit => LLTy::Void,
            TyKind::I32 => LLTy::I32,
            TyKind::Bool => LLTy::I8,
            TyKind::Array(elem_ty, n) => LLTy::Array(Rc::new(self.ty_to_llty(elem_ty)), *n),
            TyKind::Adt(name) => LLTy::Adt(Rc::clone(name)),
            TyKind::Never => LLTy::Void,
            TyKind::Ref(inner) => match &inner.kind {
                // FIXME: should be [N x i8]
                TyKind::Str => LLTy::Ptr(Rc::new(LLTy::I8)),
                _ => todo!(),
            },
            _ => panic!(),
        }
    }

    fn construct_lladt(&self, adt: &AdtDef) -> LLAdtDef {
        let mut fields = vec![];
        for (fd, fd_ty) in &adt.fields {
            fields.push((Rc::clone(fd), Rc::new(self.ty_to_llty(fd_ty))))
        }
        LLAdtDef { fields }
    }

    fn add_lladt(&mut self, name: &Rc<CanonicalPath>, lladt: LLAdtDef) {
        self.ll_adt_defs.insert(Rc::clone(name), Rc::new(lladt));
    }

    fn get_lladt(&self, name: &CanonicalPath) -> Option<Rc<LLAdtDef>> {
        self.ll_adt_defs.get(name).map(Rc::clone)
    }

    fn push_frame(&mut self, frame: Frame) {
        self.current_frame = Some(frame);
    }

    fn peek_frame_mut(&mut self) -> &mut Frame {
        self.current_frame.as_mut().unwrap()
    }

    fn peek_frame(&self) -> &Frame {
        self.current_frame.as_ref().unwrap()
    }

    fn pop_frame(&mut self) {
        if self.current_frame.is_none() {
            panic!("ICE: cannot pop the current frame");
        }
        self.current_frame = None;
    }

    /// Generate code for top-level
    fn go(&mut self, krate: &'gen Crate) -> Result<(), ()> {
        println!(r#"target triple = "x86_64-unknown-linux-gnu""#);
        println!();
        println!("declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg) #1");
        println!();

        // register all ADTs
        let mut lladts = vec![];
        for (name, adt_def) in self.ctx.get_adt_defs() {
            let lladt = self.construct_lladt(adt_def);
            lladts.push((Rc::clone(name), lladt));
        }
        for (cpath, lladt) in lladts {
            print!("%Struct.{} = type {{", cpath.demangle());
            for (i, (_, fd_llty)) in lladt.fields.iter().enumerate() {
                print!(" {}", fd_llty.to_string());
                if i != lladt.fields.len() - 1 {
                    print!(",");
                }
            }
            println!(" }}");
            self.add_lladt(&cpath, lladt);
        }

        println!();
        self.gen_crate(krate)?;

        // string literals
        for cons in &self.constants {
            println!(
                "{} = constant {} c\"{}\\00\"",
                cons.name,
                cons.llty.to_string(),
                cons.string_lit
            );
        }

        Ok(())
    }

    pub fn get_size(&self, llty: &LLTy) -> usize {
        match llty {
            LLTy::I32 => 4,
            LLTy::I8 => 1,
            LLTy::Ptr(_) => 1,
            LLTy::Array(elem_llty, n) => self.get_align(elem_llty) * n,
            LLTy::Void => panic!(),
            LLTy::Adt(name) => {
                let lladt = self.get_lladt(name).unwrap();
                self.get_lladt_size(&lladt)
            }
        }
    }

    pub fn get_lladt_size(&self, lladt: &LLAdtDef) -> usize {
        let mut ofs = 0;
        for (_, fd_llty) in &lladt.fields {
            let fd_align = self.get_align(fd_llty);
            ofs += padding_size(ofs, fd_align);
            ofs += self.get_size(fd_llty);
        }
        ofs += padding_size(ofs, self.get_lladt_align(lladt));
        ofs
    }

    pub fn get_align(&self, llty: &LLTy) -> usize {
        match llty {
            LLTy::I32 => 4,
            LLTy::I8 => 1,
            LLTy::Ptr(_) => 1,
            LLTy::Array(elem_llty, _) => self.get_align(elem_llty),
            LLTy::Void => panic!(),
            LLTy::Adt(name) => {
                let lladt = self.get_lladt(name).unwrap();
                self.get_lladt_align(&lladt)
            }
        }
    }

    pub fn get_lladt_align(&self, lladt: &LLAdtDef) -> usize {
        let mut max_align = 1;
        for (_, fd_llty) in &lladt.fields {
            let fd_align = self.get_align(fd_llty);
            if fd_align > max_align {
                max_align = fd_align;
            }
        }
        max_align
    }
}

// e.g. ofs: 1, align: 4 => 3
fn padding_size(ofs: usize, align: usize) -> usize {
    if ofs % align == 0 {
        0
    } else {
        align - (ofs % align)
    }
}
