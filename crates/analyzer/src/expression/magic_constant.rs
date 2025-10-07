use mago_atom::atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::get_class_string;
use mago_codex::ttype::get_empty_string;
use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::get_literal_string;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;
use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for MagicConstant<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let constant_type = match self {
            MagicConstant::Line(_) => {
                get_literal_int(context.source_file.line_number(self.start_position().offset()) as i64 + 1)
            }
            MagicConstant::File(_) => {
                if let Some(path) = context.source_file.path.as_deref().and_then(|p| p.to_str()) {
                    get_literal_string(atom(path))
                } else {
                    get_non_empty_string()
                }
            }
            MagicConstant::Directory(_) => {
                if let Some(path) =
                    context.source_file.path.as_deref().and_then(|p| p.parent()).and_then(|p| p.to_str())
                {
                    get_literal_string(atom(path))
                } else {
                    get_non_empty_string()
                }
            }
            MagicConstant::Namespace(_) => {
                if let Some(namespace_name) = context.scope.namespace_name() {
                    TUnion::from_atomic(TAtomic::Scalar(TScalar::String(TString::from(namespace_name.to_owned()))))
                } else {
                    get_empty_string()
                }
            }
            MagicConstant::Trait(_) => {
                if let Some(class_like) = block_context.scope.get_class_like() {
                    if class_like.kind.is_trait() {
                        TUnion::from_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::literal(
                            class_like.original_name,
                        ))))
                    } else {
                        get_empty_string()
                    }
                } else {
                    get_string()
                }
            }
            MagicConstant::Class(_) => {
                if let Some(class_like) = block_context.scope.get_class_like() {
                    if !class_like.kind.is_trait() {
                        TUnion::from_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::literal(
                            class_like.original_name,
                        ))))
                    } else {
                        get_class_string()
                    }
                } else {
                    get_string()
                }
            }
            MagicConstant::Function(_) | MagicConstant::Method(_) => {
                if block_context.scope.get_function_like().is_none() { get_string() } else { get_non_empty_string() }
            }
            MagicConstant::Property(_) => get_string(),
        };

        artifacts.set_expression_type(&self, constant_type);

        Ok(())
    }
}
