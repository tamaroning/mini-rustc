use super::frame_info::FrameInfo;
use crate::analysis::Ctxt;
use crate::ast::{BinOp, Crate, Expr, ExprKind, Func, Stmt, StmtKind, UnOp};

const PARAM_REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

pub fn codegen(ctx: &Ctxt, krate: &Crate) -> Result<(), ()> {
    let mut codegen = Codegen::new(ctx);
    codegen.codegen_crate(krate)?;
    Ok(())
}

struct Codegen<'a> {
    ctx: &'a Ctxt,
    current_frame: Option<FrameInfo<'a>>,
    next_label_id: u32,
}

impl<'a> Codegen<'a> {
    fn new(ctx: &'a Ctxt) -> Self {
        Codegen {
            ctx,
            current_frame: None,
            next_label_id: 0,
        }
    }

    fn get_new_label_id(&mut self) -> u32 {
        let id = self.next_label_id;
        self.next_label_id += 1;
        id
    }

    fn push_current_frame(&mut self, frame: FrameInfo<'a>) {
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

    fn codegen_crate(&mut self, krate: &'a Crate) -> Result<(), ()> {
        println!(".intel_syntax noprefix");
        println!(".globl main");
        for func in &krate.items {
            self.codegen_func(func)?;
        }
        Ok(())
    }

    fn codegen_func(&mut self, func: &'a Func) -> Result<(), ()> {
        let frame = FrameInfo::compute(func);
        if self.ctx.dump_enabled {
            dbg!(&frame);
        }
        self.push_current_frame(frame);

        println!("{}:", func.name.symbol);
        self.codegen_func_prologue()?;
        self.codegen_stmts(&func.body.stmts)?;
        // codegen of the last stmt results the last computation result stored in rax
        self.codegen_func_epilogue();

        self.pop_current_frame();
        Ok(())
    }

    fn codegen_func_prologue(&self) -> Result<(), ()> {
        let frame = self.get_current_frame();
        println!("\tpush rbp");
        println!("\tmov rbp, rsp");
        println!("\tsub rsp, {}", frame.size);
        for (i, (_, local)) in frame.args.iter().enumerate() {
            println!("\tmov [rbp-{}], {}", local.offset, PARAM_REGISTERS[i]);
        }
        Ok(())
    }

    fn codegen_func_epilogue(&self) {
        println!("\tmov rsp, rbp");
        println!("\tpop rbp");
        println!("\tret");
    }

    fn codegen_stmts(&mut self, stmts: &Vec<Stmt>) -> Result<(), ()> {
        for stmt in stmts {
            self.codegen_stmt(stmt)?;
        }
        Ok(())
    }

    fn codegen_stmt(&mut self, stmt: &Stmt) -> Result<(), ()> {
        match &stmt.kind {
            StmtKind::Semi(expr) => {
                self.codegen_expr(expr)?;
                // store the last result of computation to rax
                println!("\tpop rax");
                Ok(())
            }
            StmtKind::Expr(expr) => {
                self.codegen_expr(expr)?;
                // store the last result of computation to rax
                println!("\tpop rax");
                Ok(())
            }
            StmtKind::Let(_name) => Ok(()),
        }
    }

    fn codegen_expr(&mut self, expr: &Expr) -> Result<(), ()> {
        match &expr.kind {
            ExprKind::NumLit(n) => {
                println!("#lit");
                println!("\tpush {}", n);
                Ok(())
            }
            ExprKind::BoolLit(b) => {
                if *b {
                    println!("\tpush 1");
                } else {
                    println!("\tpush 0");
                }
                Ok(())
            }
            ExprKind::Unary(unop, inner_expr) => {
                println!("#unary");
                match unop {
                    UnOp::Plus => self.codegen_expr(inner_expr),
                    UnOp::Minus => {
                        // compile `-expr` as `0 - expr`
                        println!("\tpush 0");
                        self.codegen_expr(inner_expr)?;
                        println!("\tpop rdi");
                        println!("\tpop rax");
                        println!("\tsub rax, rdi");
                        println!("\tpush rax");
                        Ok(())
                    }
                }
            }
            ExprKind::Binary(binop, lhs, rhs) => {
                println!("#binary");
                self.codegen_expr(lhs)?;
                self.codegen_expr(rhs)?;
                println!("\tpop rdi");
                println!("\tpop rax");

                match binop {
                    BinOp::Add => {
                        println!("\tadd rax, rdi");
                    }
                    BinOp::Sub => {
                        println!("\tsub rax, rdi");
                    }
                    BinOp::Mul => {
                        // NOTE: Result of mul is stored to rax
                        println!("\tmul rdi");
                    }
                    BinOp::Eq => {
                        println!("\tcmp rax,rdi");
                        println!("\tsete al");
                        println!("\tmovzb rax, al");
                    }
                    _ => todo!(),
                };
                println!("\tpush rax");
                Ok(())
            }
            ExprKind::Ident(_ident) => {
                println!("#ident");
                self.codegen_lval(expr)?;
                println!("\tpop rax");
                // TODO: use al, ax, eax for type whose size is < 8
                println!("\tmov rax, [rax]");
                println!("\tpush rax");
                Ok(())
            }
            ExprKind::Assign(lhs, rhs) => {
                println!("#assign");
                self.codegen_lval(lhs)?;
                self.codegen_expr(rhs)?;
                println!("\tpop rdi");
                println!("\tpop rax");
                println!("\tmov [rax], rdi");
                // TODO: It is better not to push to stack
                // push dummy similarly to other exprs for simplicity
                println!("\tpush 99");
                Ok(())
            }
            ExprKind::Return(inner) => {
                self.codegen_expr(inner)?;
                println!("\tpop rax");
                println!("\tmov rsp, rbp");
                println!("\tpop rbp");
                println!("\tret");
                Ok(())
            }
            ExprKind::Call(ident, args) => {
                if args.len() > 6 {
                    todo!("number of args must be < 6");
                }
                for param in args {
                    self.codegen_expr(param)?;
                }
                for i in 0..args.len() {
                    println!("\tpop {}", PARAM_REGISTERS[i]);
                }
                println!("\tcall {}", ident.symbol);
                println!("\tpush rax");
                Ok(())
            }
            ExprKind::Block(block) => {
                self.codegen_stmts(&block.stmts)?;
                // codegen_stmt results rax with the last result of computation in it
                // so push it to stack
                println!("\tpush rax");
                Ok(())
            }
            ExprKind::If(cond, then, els) => {
                let label_id = self.get_new_label_id();
                self.codegen_expr(cond)?;
                println!("\tpop rax");
                println!("\tcmp rax, 0");
                if els.is_some() {
                    println!("\tje .Lelse{label_id}");
                } else {
                    println!("\tje .Lend{label_id}");
                }
                self.codegen_expr(then)?;

                if let Some(els) = els {
                    println!("\tjmp .Lend{label_id}");
                    println!(".Lelse{label_id}:");
                    self.codegen_expr(els)?;
                }
                println!(".Lend{label_id}:");
                Ok(())
            }
            ExprKind::Index(ident, index) => {
                todo!()
            }
        }
    }

    fn codegen_lval(&self, expr: &Expr) -> Result<(), ()> {
        let ExprKind::Ident(ident) = &expr.kind else {
            eprintln!("ICE: Cannot codegen {:?} as lval", expr);
            return Err(());
        };
        // Try to find ident in all locals
        if let Some(local) = self.get_current_frame().locals.get(&ident.symbol) {
            println!("#lval");
            println!("\tmov rax, rbp");
            println!("\tsub rax, {}", local.offset);
            println!("\tpush rax");
            Ok(())
        }
        // Try to find ident in all args
        else if let Some(arg) = self.get_current_frame().args.get(&ident.symbol) {
            println!("#lval");
            println!("\tmov rax, rbp");
            println!("\tsub rax, {}", arg.offset);
            println!("\tpush rax");
            Ok(())
        } else {
            eprintln!("Unknwon identifier: {}", ident.symbol);
            Err(())
        }
    }
}
