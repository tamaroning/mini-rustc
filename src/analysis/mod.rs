#[derive(Debug)]
pub struct Ctxt {
    pub dump_enabled: bool,
}

impl<'ctx> Ctxt {
    pub fn new(dump_enabled: bool) -> Self {
        Ctxt { dump_enabled }
    }
}
