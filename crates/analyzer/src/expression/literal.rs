use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_codex::ttype::get_empty_string;
use mago_codex::ttype::get_false;
use mago_codex::ttype::get_int_or_float;
use mago_codex::ttype::get_literal_float;
use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::get_literal_string;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_true;
use mago_syntax::ast::Literal;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Literal<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        _context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if let Literal::String(literal_string) = self
            && let Some(value) = literal_string.value
            && let Some((class_part, method_part)) = value.split_once("::")
            && !class_part.is_empty()
            && !method_part.is_empty()
            && !method_part.contains("::")
        {
            artifacts.symbol_references.add_reference_to_class_member(
                &block_context.scope,
                (ascii_lowercase_atom(class_part), ascii_lowercase_atom(method_part)),
                false,
            );
        }

        artifacts.set_expression_type(
            &self,
            match self {
                Literal::String(literal_string) => match literal_string.value {
                    Some(value) => get_literal_string(atom(value)),
                    None => {
                        if literal_string.raw.len() >= 3 {
                            get_non_empty_string()
                        } else {
                            get_empty_string()
                        }
                    }
                },
                Literal::Integer(literal_integer) => match literal_integer.value {
                    Some(value) => get_literal_int(value as i64),
                    None => get_int_or_float(),
                },
                Literal::Float(literal_float) => get_literal_float(*literal_float.value),
                Literal::True(_) => get_true(),
                Literal::False(_) => get_false(),
                Literal::Null(_) => get_null(),
            },
        );

        Ok(())
    }
}
