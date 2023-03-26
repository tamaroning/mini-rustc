use std::collections::HashMap;

#[derive(Debug)]
pub struct Ctxt<'ctx> {
    ty_mapping: HashMap<&'ctx str, Ty>,
}

#[derive(Debug)]
pub enum Ty {
    I32,
}

impl<'ctx> Ctxt<'ctx> {
    pub fn new() -> Self {
        Ctxt {
            ty_mapping: HashMap::new(),
        }
    }

    pub fn set_type(&mut self, name: &'ctx str, ty: Ty) {
        let t = self.ty_mapping.insert(name, ty);
        if t.is_some() {
            panic!("ICE: dulplicated identifier? {name}");
        }
    }

    pub fn lookup_type(&self, name: &str) -> Option<&Ty> {
        self.ty_mapping.get(name)
    }
}
