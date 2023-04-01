mod codegen;
mod frame_info;

use self::codegen::codegen;
use crate::middle::Ctxt;
use crate::ast::{self};

pub fn compile(ctx: &Ctxt, krate: &ast::Crate) -> Result<(), ()> {
    codegen(ctx, krate)?;

    Ok(())
}
