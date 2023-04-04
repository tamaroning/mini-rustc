use super::frame_info::FrameInfo;
use crate::ast::{
    BinOp, Crate, Expr, ExprKind, Func, Ident, ItemKind, LetStmt, Stmt, StmtKind, UnOp,
};
use crate::middle::ty::{AdtDef, Ty};
use crate::middle::Ctxt;
use crate::resolve::BindingKind;
use std::collections::HashMap;

const PARAM_REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

pub fn codegen(ctx: &mut Ctxt, krate: &Crate) -> Result<(), ()> {
    let mut codegen = Codegen::new(ctx);
    codegen.go(krate)?;
    Ok(())
}

// TODO:
// add func to NameBinding mappings
// add NameBinding to LocalInfo mappings
struct Codegen<'a> {
    ctx: &'a mut Ctxt,
    current_frame: Option<FrameInfo>,
    // String literal to label mappings
    // "some_lit" => .LCN
    str_label_mappings: HashMap<&'a String, String>,
    next_label_id: u32,
}

impl<'a> Codegen<'a> {
    fn new(ctx: &'a mut Ctxt) -> Self {
        Codegen {
            ctx,
            current_frame: None,
            str_label_mappings: HashMap::new(),
            next_label_id: 0,
        }
    }

    fn get_new_label_id(&mut self) -> u32 {
        let id = self.next_label_id;
        self.next_label_id += 1;
        id
    }

    fn push_current_frame(&mut self, frame: FrameInfo) {
        self.current_frame = Some(frame);
    }

    fn get_current_frame(&self) -> &FrameInfo {
        let Some(f) = &self.current_frame else {
            panic!("ICE");
        };
        f
    }

    fn pop_current_frame(&mut self) {
        if self.current_frame.is_none() {
            panic!("ICE: cannot pop the current frame");
        }
        self.current_frame = None;
    }

    fn go(&mut self, krate: &'a Crate) -> Result<(), ()> {
        println!(".intel_syntax noprefix");
        println!(".globl main");
        self.codegen_crate(krate)?;
        for (str, label) in self.str_label_mappings.iter() {
            println!("{label}:");
            println!("\t.ascii \"{str}\"");
            println!("\t.zero 1");
        }
        Ok(())
    }

    fn codegen_crate(&mut self, krate: &'a Crate) -> Result<(), ()> {
        for item in &krate.items {
            match &item.kind {
                ItemKind::Func(func) => {
                    self.codegen_func(func)?;
                }
                ItemKind::Struct(_) => (),
                ItemKind::ExternBlock(_) => (),
            }
        }
        Ok(())
    }

    fn codegen_func(&mut self, func: &'a Func) -> Result<(), ()> {
        // do not generate code for the func if it does not have its body
        if func.body.is_none() {
            return Ok(());
        }

        let frame = FrameInfo::compute(self.ctx, func);
        if self.ctx.dump_enabled {
            dbg!(&frame);
        }
        self.push_current_frame(frame);

        println!("{}:", func.name.symbol);
        self.codegen_func_prologue()?;
        if let Some(body) = &func.body {
            for stmt in &body.stmts {
                self.codegen_stmt(stmt)?;
            }
        }
        // codegen of the last stmt results the last computation result stored in rax
        self.codegen_func_epilogue(func);

        self.pop_current_frame();
        Ok(())
    }

    fn codegen_func_prologue(&self) -> Result<(), ()> {
        let frame = self.get_current_frame();
        println!("\tpush rbp");
        println!("\tmov rbp, rsp");
        println!("\tsub rsp, {}", frame.size);
        for (i, (_, local)) in frame
            .locals
            .iter()
            .filter(|(binding, _)| binding.kind == BindingKind::Arg)
            .enumerate()
        {
            // FIXME: size > 8 and size == 0?
            println!("\tmov rax, {} # load {i}th param", PARAM_REGISTERS[i]);
            println!("\tmov rdi, rbp");
            println!("\tsub rdi, {}", local.offset);
            self.load_ax_to_rdi(local.size);
        }
        Ok(())
    }

    fn codegen_func_epilogue(&self, func: &'a Func) {
        // FIXME: remove this?
        if let Some(body) = &func.body {
            let block_ty = self.ctx.get_block_type(body);
            if *block_ty == Ty::Unit {
                println!("\tmov rax, 0");
            }
        }
        println!("\tmov rsp, rbp");
        println!("\tpop rbp");
        println!("\tret");
    }

    fn codegen_stmt(&mut self, stmt: &'a Stmt) -> Result<StoreKind, ()> {
        println!("# Starts stmt `{}`", stmt.span.to_snippet());
        let store_kind = match &stmt.kind {
            StmtKind::Semi(expr) => {
                let store_kind = self.codegen_expr(expr)?;

                // In case of struct type, pop stack to clean it.
                /* TODO:
                if store_kind == StoreKind::Stack {
                    let ty = self.ctx.get_type(expr.id);
                    // TODO: clean up array
                    if ty.is_adt() {
                        self.clean_adt_on_stack(ty.get_adt_name().unwrap());
                    }
                }
                */
                StoreKind::None
            }
            StmtKind::Expr(expr) => self.codegen_expr(expr)?,
            StmtKind::Let(LetStmt { ident, ty, init }) => {
                if let Some(init) = init {
                    self.codegen_assign_local_var(ident, ty, init)?;
                }
                StoreKind::None
            }
        };
        println!("# Finished stmt `{}`", stmt.span.to_snippet());
        Ok(store_kind)
    }

    /// Generate code for expression.
    /// Result is stored to al, eax, or rax. In case of al and eax, rax is zero-extended with al, or eax.
    /// If size of expr is = 0, rax is not set.
    /// If size of expr is > 0 and <= 8, rax is set.
    /// If size of expr is > 8, all of its fields are pushed to the stack.
    /// TODO: store kind
    fn codegen_expr(&mut self, expr: &'a Expr) -> Result<StoreKind, ()> {
        println!("# Starts expr `{}`", expr.span.to_snippet());
        match &expr.kind {
            ExprKind::NumLit(n) => {
                println!("\tmov rax, {}", n);
            }
            ExprKind::BoolLit(b) => {
                if *b {
                    println!("\tmov rax, 1");
                } else {
                    println!("\tmov rax, 0");
                }
            }
            ExprKind::StrLit(s) => {
                let label = format!(".LC{}", self.get_new_label_id());
                println!("\tmov eax, OFFSET FLAT:{label} # static str");
                // register the constant label
                if self.str_label_mappings.get(s).is_none() {
                    self.str_label_mappings.insert(s, label);
                }
            }
            ExprKind::Unit => {
                // main returns unit, so set 0 to rax
                println!("\tmov rax, 0");
                return Ok(StoreKind::None);
            }
            ExprKind::Unary(unop, inner_expr) => {
                match unop {
                    UnOp::Plus => {
                        let s = self.codegen_expr(inner_expr)?;
                        assert!(s == StoreKind::Rax);
                    }
                    UnOp::Minus => {
                        // compile `-expr` as `0 - expr`
                        let s = self.codegen_expr(inner_expr)?;
                        assert!(s == StoreKind::Rax);
                        println!("\tmov rdi, rax");
                        println!("\tmov rax, 0");
                        println!("\tsub rax, rdi");
                    }
                }
            }
            ExprKind::Binary(binop, lhs, rhs) => {
                // use rax and rdi if rhs/lhs is size of 64bit
                let ax = "eax";
                let di = "edi";
                let s = self.codegen_expr(lhs)?;
                assert!(s == StoreKind::Rax);
                self.push();
                let s = self.codegen_expr(rhs)?;
                assert!(s == StoreKind::Rax);
                self.push();
                self.pop("rdi");
                self.pop("rax");

                match binop {
                    BinOp::Add => {
                        println!("\tadd {}, {}", ax, di);
                    }
                    BinOp::Sub => {
                        println!("\tsub {}, {}", ax, di);
                    }
                    BinOp::Mul => {
                        // NOTE: Result is stored in rax
                        println!("\tmul {}", di);
                    }
                    BinOp::Eq => {
                        println!("\tcmp {}, {}", ax, di);
                        println!("\tsete al");
                        // zero extended to rax later
                    }
                    _ => todo!(),
                };
            }
            ExprKind::Ident(_) | ExprKind::Index(_, _) | ExprKind::Field(_, _) => {
                println!("#ident or index");
                self.codegen_addr(expr)?;
                println!("\tmov rax, [rax]");
                // TODO: store kind
            }
            ExprKind::Assign(lhs, rhs) => {
                self.codegen_assign(lhs, rhs)?;
                return Ok(StoreKind::None);
            }
            ExprKind::Return(inner) => {
                let s = self.codegen_expr(inner)?;
                if s != StoreKind::Rax && s != StoreKind::None {
                    // TODO: return struct and arrays
                    todo!()
                }
                println!("\tmov rsp, rbp");
                // TODO: remove this?
                let inner_ty = self.ctx.get_type(inner.id);
                if *inner_ty == Ty::Unit {
                    println!("\tmov rax, 0");
                }
                println!("\tpop rbp");
                println!("\tret");
                return Ok(StoreKind::None);
            }
            ExprKind::Call(func, args) => {
                if args.len() > 6 {
                    todo!("number of args must be < 6");
                }
                for param in args {
                    // TODO: pass struct param via stack
                    // p16. https://www.uclibc.org/docs/psABI-x86_64.pdf
                    let s = self.codegen_expr(param)?;
                    // TODO: StoreKind::None
                    if s != StoreKind::Rax {
                        todo!();
                    }
                    self.push();
                }
                for i in 0..args.len() {
                    self.pop(PARAM_REGISTERS[i]);
                }
                let name = self.retrieve_name(func)?;
                // FIXME: To support va_args, set 0 to rax
                println!("\tmov eax, 0");
                println!("\tcall {}", name.symbol);
                // TODO: StoreKind::Stack?
            }
            ExprKind::Block(block) => {
                let mut store_kind = StoreKind::Rax;
                for stmt in &block.stmts {
                    store_kind = self.codegen_stmt(stmt)?;
                }
                return Ok(store_kind);
            }
            ExprKind::If(cond, then, els) => {
                let label_id = self.get_new_label_id();
                let s = self.codegen_expr(cond)?;
                assert!(s == StoreKind::Rax);
                println!("\tcmp rax, 0");
                if els.is_some() {
                    println!("\tje .Lelse{label_id}");
                } else {
                    println!("\tje .Lend{label_id}");
                }
                let store_kind = self.codegen_expr(then)?;

                if let Some(els) = els {
                    println!("\tjmp .Lend{label_id}");
                    println!(".Lelse{label_id}:");
                    let els_store_kind = self.codegen_expr(els)?;
                    assert!(store_kind == els_store_kind);
                }
                println!(".Lend{label_id}:");
                return Ok(store_kind);
            }
            // storategy of chibicc
            // load(ty): load value to rax (array, struct: No)
            //   ref: https://github.com/rui314/chibicc/blob/90d1f7f199cc55b13c7fdb5839d1409806633fdb/codegen.c#L186
            // store(ty): store [rax] to an address that the stack top is pointing to (array, struct: OK)
            //   ref: https://github.com/rui314/chibicc/blob/90d1f7f199cc55b13c7fdb5839d1409806633fdb/codegen.c#L233-L238
            //
            // But it seems that chibicc does not support expressions whose size is > 8 (e.g. struct expr, array expr)
            //   ref: https://github.com/rui314/chibicc/blob/main/test/struct.c
            //
            // Solution I have come up with:
            //   1. Allocate stack frames every time AST nodes which have struct or array type but are not variables are found.
            //   2. Then we can store a value to them using address where allocated data are located.
            ExprKind::Struct(ident, fds) => {
                // TODO:
                todo!()
                /*
                let _adt = self.ctx.lookup_adt_def(&ident.symbol).unwrap();
                // starts pushing from the first field
                for (_, fd) in fds {
                    // TODO: deal with order
                    self.codegen_expr(fd)?;
                    let fd_ty = self.ctx.get_type(fd.id);
                    let fd_size = self.ctx.get_size(&fd_ty);
                    if !matches!(*fd_ty, Ty::Adt(_) | Ty::Array(_, _)) && fd_size != 0 {
                        self.push();
                    }
                }
                return Ok(StoreKind::Stack);
                */
            }
            ExprKind::Array(elems) => {
                // TODO:
                // ref:
                todo!()
                // starts pushing from the first element
                /*
                for e in elems {
                    self.codegen_expr(e)?;
                    let elem_ty = self.ctx.get_type(e.id);
                    let elem_size = self.ctx.get_size(&elem_ty);
                    // TODO: size?
                    if !matches!(*elem_ty, Ty::Adt(_) | Ty::Array(_, _)) && elem_size != 0 {
                        self.push();
                    }
                }
                return Ok(StoreKind::Stack);*/
            }
        }

        // Extract the significant bits
        let ty = self.ctx.get_type(expr.id);
        match &*ty {
            Ty::Bool => {
                println!("\tmovzx rax, al");
            }
            Ty::I32 => {
                println!("\tmovsx rax, eax");
            }
            _ => (),
        }

        println!("# Finishes expr `{}`", expr.span.to_snippet());
        Ok(StoreKind::Rax)
    }

    /// Load address to rax
    fn codegen_addr(&mut self, expr: &'a Expr) -> Result<(), ()> {
        match &expr.kind {
            ExprKind::Ident(ident) => {
                self.codegen_addr_local_var(ident)?;
                Ok(())
            }
            ExprKind::Index(array, index) => {
                let elem_ty_size = self.ctx.get_size(&self.ctx.get_type(expr.id));
                self.codegen_addr(array)?;
                self.push();
                let s = self.codegen_expr(index)?;
                assert!(s == StoreKind::Rax);
                self.push();
                self.pop("rdi"); // rdi <- index
                println!("\tmov rax, {}", elem_ty_size); // rax <- size_of(size)
                println!("\tmul rdi"); // rax <- index * size_of(elem)
                self.pop("rdi"); // rdi <- base_addr
                println!("\tadd rax, rdi"); // rax <- base_addr + index * size_of(elem)
                Ok(())
            }
            ExprKind::Field(recv, fd) => {
                self.codegen_addr(recv)?;

                let offs = self
                    .ctx
                    .get_field_offset(
                        self.ctx.get_type(recv.id).get_adt_name().unwrap(),
                        &fd.symbol,
                    )
                    .unwrap();
                println!("\tadd rax, {}", offs);
                Ok(())
            }
            _ => {
                eprintln!("ICE: Cannot codegen {:?} as lval", expr);
                Err(())
            }
        }
    }

    /// Load address to rax
    fn codegen_addr_local_var(&mut self, ident: &'a Ident) -> Result<(), ()> {
        // Try to find ident in all locals
        if let Some(binding) = self.ctx.resolver.resolve_ident(ident) {
            let local = self.get_current_frame().locals.get(&binding).unwrap();
            println!("\tmov rax, rbp");
            println!("\tsub rax, {}", local.offset);
            Ok(())
        } else {
            eprintln!("Unknwon identifier: {}", ident.symbol);
            Err(())
        }
    }

    // FIXME: sync with `codegen_assign`
    fn codegen_assign_local_var(
        &mut self,
        name: &'a Ident,
        ty: &Ty,
        expr: &'a Expr,
    ) -> Result<(), ()> {
        let size = self.ctx.get_size(ty);

        let store_kind = self.codegen_expr(expr)?;
        match store_kind {
            StoreKind::Stack => {
                let flatten_fields = if let Ty::Adt(name) = ty {
                    let adt = self.ctx.lookup_adt_def(name).unwrap();
                    self.ctx.flatten_struct(&adt)
                } else if let Ty::Array(elem_ty, elem_num) = ty {
                    self.ctx.flatten_array(elem_ty, *elem_num)
                } else {
                    panic!("ICE");
                };
                for (fd_ty, ofs) in flatten_fields.iter().rev() {
                    self.codegen_addr_local_var(name)?;
                    println!("\tmov rdi, rax");
                    println!("\tadd rdi, {ofs}");
                    self.pop("rax"); // rax <- addr
                    let fd_size = self.ctx.get_size(fd_ty);
                    self.load_ax_to_rdi(fd_size);
                }
            }
            StoreKind::Rax => {
                // push lhs to stack
                self.push();
                self.codegen_addr_local_var(name)?;
                self.push();
                self.pop("rdi"); // rdi <- addr of lhs
                self.pop("rax"); // rax <- rhs
                self.load_ax_to_rdi(size);
            }
            StoreKind::None => {
                // do nothing
            }
        }
        Ok(())
    }

    // FIXME: sync with `codegen_assign_local_var`
    fn codegen_assign(&mut self, lhs: &'a Expr, rhs: &'a Expr) -> Result<(), ()> {
        let ty = self.ctx.get_type(rhs.id);
        let size = self.ctx.get_size(&ty);

        let store_kind = self.codegen_expr(rhs)?;
        match store_kind {
            StoreKind::Stack => {
                let flatten_fields = if let Ty::Adt(name) = &*ty {
                    let adt = self.ctx.lookup_adt_def(name).unwrap();
                    self.ctx.flatten_struct(&adt)
                } else if let Ty::Array(elem_ty, elem_num) = &*ty {
                    self.ctx.flatten_array(elem_ty, *elem_num)
                } else {
                    panic!("ICE");
                };
                for (fd_ty, ofs) in flatten_fields.iter().rev() {
                    self.codegen_addr(lhs)?;
                    println!("\tmov rdi, rax");
                    println!("\tadd rdi, {ofs}");
                    self.pop("rax");
                    let fd_size = self.ctx.get_size(fd_ty);
                    self.load_ax_to_rdi(fd_size);
                }
            }
            StoreKind::Rax => {
                self.push();
                self.codegen_addr(lhs)?;
                self.push();
                self.pop("rdi"); // rdi <- lhs
                self.pop("rax"); // rax <- rhs
                self.load_ax_to_rdi(size);
            }
            StoreKind::None => {
                // do nothing
            }
        }
        Ok(())
    }

    fn load_ax_to_rdi(&self, size: usize) {
        match size {
            0 => (),
            1 => println!("\tmov BYTE PTR [rdi], al"),
            2..=4 => println!("\tmov DWORD PTR [rdi], eax"),
            5..=8 => println!("\tmov QWORD PTR [rdi], rax"),
            _ => panic!("ICE"),
        }
    }

    fn retrieve_name<'b>(&'b self, expr: &'b Expr) -> Result<&Ident, ()> {
        match &expr.kind {
            ExprKind::Ident(ident) => Ok(ident),
            _ => Err(()),
        }
    }

    fn push(&self) {
        println!("\tpush rax");
    }

    fn pop(&self, reg: &str) {
        println!("\tpop {}", reg);
    }

    // FIXME:
    fn clean_adt_on_stack(&mut self, adt_name: &String) {
        let size = self.ctx.get_adt_info(adt_name).size;
        // FIXME: correct?
        let pop_rax_time = size / 8;
        for _ in 0..pop_rax_time {
            self.pop("rax");
        }
    }
}

#[derive(PartialEq, Eq)]
enum StoreKind {
    Stack,
    Rax,
    None,
}
