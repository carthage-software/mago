use mago_codex::ttype::get_false;
use mago_codex::ttype::get_literal_float;
use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::get_literal_string;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_true;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl Analyzable for Literal {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        _block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        artifacts.set_expression_type(
            &self,
            match self {
                Literal::String(literal_string) => match literal_string.value.as_deref() {
                    Some(value) => get_literal_string(value.to_owned()),
                    None => {
                        let raw = context.interner.lookup(&literal_string.raw);

                        if raw.len() >= 3 { get_non_empty_string() } else { get_literal_string(String::new()) }
                    }
                },
                Literal::Integer(literal_integer) => get_literal_int(literal_integer.value as i64),
                Literal::Float(literal_float) => get_literal_float(*literal_float.value),
                Literal::True(_) => get_true(),
                Literal::False(_) => get_false(),
                Literal::Null(_) => get_null(),
            },
        );

        Ok(())
    }
}
