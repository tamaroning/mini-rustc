mod codegen_crate;
mod codegen_expr;
mod codegen_utils;
mod frame;
mod llvm;

use self::frame::Frame;
use self::llvm::*;
use crate::ast::Crate;
use crate::middle::ty::{AdtDef, Ty};
use crate::middle::Ctxt;
use std::collections::HashMap;
use std::rc::Rc;

pub fn compile(ctx: &mut Ctxt, krate: &Crate) -> Result<(), ()> {
    codegen(ctx, krate)?;

    Ok(())
}

pub fn codegen(ctx: &mut Ctxt, krate: &Crate) -> Result<(), ()> {
    let mut codegen = Codegen::new(ctx);
    codegen.go(krate)?;
    Ok(())
}

pub struct Codegen<'a> {
    ctx: &'a mut Ctxt,
    current_frame: Option<Frame>,
    ll_adt_defs: HashMap<Rc<String>, Rc<LLAdtDef>>,
    constants: Vec<Rc<LLConst>>,
    next_str_id: usize,
}

impl<'a> Codegen<'a> {
    fn new(ctx: &'a mut Ctxt) -> Self {
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
        match ty {
            Ty::Unit => LLTy::Void,
            Ty::I32 => LLTy::I32,
            Ty::Bool => LLTy::I8,
            Ty::Array(elem_ty, n) => LLTy::Array(Rc::new(self.ty_to_llty(elem_ty)), *n),
            Ty::Adt(name) => LLTy::Adt(Rc::clone(name)),
            Ty::Never => LLTy::Void,
            Ty::Ref(_, inner) => match &**inner {
                // FIXME: should be [N x i8]
                Ty::Str => LLTy::Ptr(Rc::new(LLTy::I8)),
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

    fn add_lladt(&mut self, name: &Rc<String>, lladt: LLAdtDef) {
        self.ll_adt_defs.insert(Rc::clone(name), Rc::new(lladt));
    }

    fn get_lladt(&self, name: &Rc<String>) -> Option<Rc<LLAdtDef>> {
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
    fn go(&mut self, krate: &'a Crate) -> Result<(), ()> {
        println!(r#"target triple = "x86_64-unknown-linux-gnu""#);
        println!();

        // register all ADTs
        let mut lladts = vec![];
        for (name, adt_def) in self.ctx.get_adt_defs() {
            let lladt = self.construct_lladt(adt_def);
            lladts.push((Rc::clone(name), lladt));
        }
        for (name, lladt) in lladts {
            print!("%Struct.{} = type {{", name);
            for (i, (_, fd_llty)) in lladt.fields.iter().enumerate() {
                print!(" {}", fd_llty.to_string());
                if i != lladt.fields.len() - 1 {
                    print!(",");
                }
            }
            println!(" }}");
            self.add_lladt(&name, lladt);
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
}
