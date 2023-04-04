mod codegen;
mod frame_info;

use self::codegen::codegen;
use crate::ast::{self};
use crate::middle::Ctxt;

pub fn compile(ctx: &mut Ctxt, krate: &ast::Crate) -> Result<(), ()> {
    codegen(ctx, krate)?;

    Ok(())
}
