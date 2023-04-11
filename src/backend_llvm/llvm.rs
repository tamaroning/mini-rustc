use std::rc::Rc;

use crate::resolve::CanonicalPath;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LLTy {
    Void,
    I8,
    I32,
    Ptr(Rc<LLTy>),
    Array(Rc<LLTy>, usize),
    Adt(Rc<CanonicalPath>),
}

impl LLTy {
    pub fn to_string(&self) -> String {
        match self {
            LLTy::Void => "void".to_string(),
            LLTy::I8 => "i8".to_string(),
            LLTy::I32 => "i32".to_string(),
            LLTy::Ptr(inner) => format!("{}*", inner.to_string()),
            LLTy::Array(elem_ty, n) => format!("[{} x {}]", n, elem_ty.to_string()),
            LLTy::Adt(name) => format!("%Struct.{}", name.demangle()),
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, LLTy::I32)
    }

    pub fn is_signed_integer(&self) -> bool {
        matches!(self, LLTy::I32)
    }

    pub fn peel_ptr(&self) -> Option<Rc<LLTy>> {
        match self {
            LLTy::Ptr(inner) => Some(Rc::clone(inner)),
            _ => None,
        }
    }

    pub fn get_adt_cpath(&self) -> Option<Rc<CanonicalPath>> {
        match self {
            LLTy::Adt(name) => Some(Rc::clone(name)),
            _ => None,
        }
    }

    pub fn get_element_type(&self) -> Option<Rc<LLTy>> {
        match self {
            LLTy::Array(elem, _) => Some(Rc::clone(elem)),
            _ => None,
        }
    }

    pub fn is_void(&self) -> bool {
        matches!(self, LLTy::Void)
    }

    // ADTs and arrays
    // parameters of these types are passed by pointer
    pub fn eval_to_ptr(&self) -> bool {
        matches!(*self, LLTy::Adt(_) | LLTy::Array(_, _))
    }
}

pub enum LLValue {
    Reg(Rc<LLReg>),
    Imm(LLImm),
    PtrConst(Rc<LLConst>),
}

impl LLValue {
    pub fn to_string(&self) -> String {
        match self {
            LLValue::Reg(reg) => reg.name.clone(),
            LLValue::Imm(imm) => imm.to_string(),
            LLValue::PtrConst(cons) => cons.name.clone(),
        }
    }

    pub fn llty(&self) -> Rc<LLTy> {
        match self {
            LLValue::Reg(reg) => Rc::clone(&reg.llty),
            LLValue::Imm(imm) => imm.llty(),
            // `[N x i8]` => i8*
            LLValue::PtrConst(cons) => Rc::new(LLTy::Ptr(cons.llty.get_element_type().unwrap())),
        }
    }

    pub fn to_string_with_type(&self) -> String {
        match self {
            LLValue::Reg(reg) => reg.to_string_with_type(),
            LLValue::Imm(imm) => imm.to_string_with_type(),
            LLValue::PtrConst(cons) => format!("ptr {}", cons.name),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LLReg {
    pub name: String,
    pub llty: Rc<LLTy>,
}

impl LLReg {
    pub fn new(name: String, llty: Rc<LLTy>) -> Rc<Self> {
        Rc::new(LLReg { name, llty })
    }

    pub fn to_string_with_type(&self) -> String {
        format!("{} {}", self.llty.to_string(), self.name)
    }
}

pub enum LLImm {
    I32(i32),
    I8(i8),
    Void,
}

impl LLImm {
    pub fn to_string(&self) -> String {
        match self {
            LLImm::I32(n) => format!("{n}"),
            LLImm::I8(n) => format!("{n}"),
            LLImm::Void => "void".to_string(),
        }
    }

    pub fn to_string_with_type(&self) -> String {
        match self {
            LLImm::I32(n) => format!("i32 {n}"),
            LLImm::I8(n) => format!("i8 {n}"),
            LLImm::Void => "void".to_string(),
        }
    }

    pub fn llty(&self) -> Rc<LLTy> {
        Rc::new(match self {
            LLImm::I32(_) => LLTy::I32,
            LLImm::I8(_) => LLTy::I8,
            LLImm::Void => LLTy::Void,
        })
    }
}

pub struct LLAdtDef {
    pub fields: Vec<(Rc<String>, Rc<LLTy>)>,
}

impl LLAdtDef {
    pub fn get_field_index(&self, field: &String) -> Option<usize> {
        let f = self
            .fields
            .iter()
            .enumerate()
            .find(|(_, (fd, _))| **fd == *field);
        f.map(|i| i.0)
    }
}

pub struct LLConst {
    pub name: String,
    pub llty: Rc<LLTy>,
    pub string_lit: String,
}
