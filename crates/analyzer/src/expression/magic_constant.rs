use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::union::TUnion;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl Analyzable for MagicConstant {
    fn analyze(
        &self,
        context: &mut Context<'_>,
        block_context: &mut BlockContext,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut potentially_undefined = false;
        let constant_scalar = match self {
            MagicConstant::Line(_) => TScalar::Integer(TInteger::non_negative()),
            MagicConstant::File(_) => TScalar::String(TString::non_empty()),
            MagicConstant::Directory(_) => TScalar::String(TString::non_empty()),
            MagicConstant::Namespace(_) => {
                if let Some(namespace_name) = context.scope.namespace_name() {
                    TScalar::String(TString::from(namespace_name))
                } else {
                    TScalar::String(TString::from(""))
                }
            }
            MagicConstant::Trait(_) => {
                if let Some(class_like) = block_context.scope.get_class_like() {
                    if class_like.kind.is_trait() {
                        TScalar::ClassLikeString(TClassLikeString::literal(class_like.original_name))
                    } else {
                        TScalar::String(TString::from(""))
                    }
                } else {
                    potentially_undefined = true;

                    TScalar::ClassLikeString(TClassLikeString::any(TClassLikeStringKind::Trait))
                }
            }
            MagicConstant::Class(_) => {
                if let Some(class_like) = block_context.scope.get_class_like() {
                    if !class_like.kind.is_trait() {
                        TScalar::ClassLikeString(TClassLikeString::literal(class_like.original_name))
                    } else {
                        TScalar::ClassLikeString(TClassLikeString::any(TClassLikeStringKind::Class))
                    }
                } else {
                    potentially_undefined = true;

                    TScalar::ClassLikeString(TClassLikeString::any(TClassLikeStringKind::Class))
                }
            }
            MagicConstant::Function(_) | MagicConstant::Method(_) => {
                if block_context.scope.get_function_like().is_none() {
                    potentially_undefined = true;
                }

                TScalar::String(TString::general_with_props(false, true, true))
            }
            MagicConstant::Property(_) => {
                potentially_undefined = true;

                TScalar::String(TString::general_with_props(false, true, true))
            }
        };

        let mut constant_types = vec![TAtomic::Scalar(constant_scalar)];
        if potentially_undefined {
            constant_types.push(TAtomic::Scalar(TScalar::String(TString::from(""))));
        }

        artifacts.set_expression_type(&self, TUnion::new(constant_types));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_analysis;

    test_analysis! {
        name = non_empty_string_magic_constant,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-string $s
             * @return non-empty-string
             */
            function i_take_non_empty_string(string $s): string
            {
                return $s;
            }

            i_take_non_empty_string(__FILE__);
            i_take_non_empty_string(__DIR__);
        "#},
    }
}
