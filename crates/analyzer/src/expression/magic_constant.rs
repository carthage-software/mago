use mago_codex::ttype::get_int;
use mago_codex::ttype::get_string;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl Analyzable for MagicConstant {
    fn analyze(
        &self,
        _context: &mut Context<'_>,
        _block_context: &mut BlockContext,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        artifacts.set_expression_type(
            &self,
            match self {
                MagicConstant::Line(_) => get_int(),
                MagicConstant::File(_) => get_string(),
                MagicConstant::Directory(_) => get_string(),
                MagicConstant::Trait(_) => get_string(),
                MagicConstant::Method(_) => get_string(),
                MagicConstant::Function(_) => get_string(),
                MagicConstant::Property(_) => get_string(),
                MagicConstant::Namespace(_) => get_string(),
                MagicConstant::Class(_) => get_string(),
            },
        );

        Ok(())
    }
}
