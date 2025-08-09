use std::rc::Rc;

use mago_codex::class_or_interface_exists;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::Code;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl Analyzable for Clone {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.object.analyze(context, block_context, artifacts)?;

        let Some(object_type) = artifacts.get_rc_expression_type(&self.object).cloned() else {
            return Ok(());
        };

        let mut invalid_clone_atomics = vec![];
        let mut has_mixed_type = false;
        let mut has_cloneable_object = false;

        let mut atomic_types = object_type.types.iter().collect::<Vec<_>>();
        loop {
            let Some(atomic_type) = atomic_types.pop() else {
                break;
            };

            match atomic_type {
                TAtomic::Object(object) => match object {
                    TObject::Any => {
                        has_cloneable_object = true;
                    }
                    TObject::Enum(_) => {
                        invalid_clone_atomics.push(atomic_type);
                    }
                    TObject::Named(named_object) => {
                        if !class_or_interface_exists(context.codebase, context.interner, &named_object.name) {
                            invalid_clone_atomics.push(atomic_type);
                        } else {
                            has_cloneable_object = true;
                        }
                    }
                },
                TAtomic::GenericParameter(parameter) => {
                    atomic_types.extend(parameter.constraint.types.iter());
                }
                TAtomic::Mixed(_) => {
                    has_mixed_type = true;
                }
                TAtomic::Scalar(scalar) if scalar.is_false() && object_type.ignore_falsable_issues => {
                    continue;
                }
                TAtomic::Null | TAtomic::Void if object_type.ignore_nullable_issues => {
                    continue;
                }
                TAtomic::Callable(callable) if callable.get_signature().is_none_or(|s| s.is_closure()) => {
                    has_cloneable_object = true;
                    continue;
                }
                _ => {
                    invalid_clone_atomics.push(atomic_type);
                }
            }
        }

        if has_mixed_type {
            context.collector.report_with_code(
                Code::MIXED_CLONE,
                Issue::warning("Cannot statically verify `clone` on a `mixed` type.")
                    .with_annotation(Annotation::primary(self.object.span()).with_message(format!(
                        "This expression has type `{}`, which could be a non-object at runtime.",
                        object_type.get_id(Some(context.interner))
                    )))
                    .with_note("Cloning requires the value to be an object. Using `clone` on a non-object will cause a fatal error.")
                    .with_help("Use type hints or `is_object()` checks to ensure the value is an object before cloning."),
            );
        }

        if !invalid_clone_atomics.is_empty() {
            let invalid_types_str =
                invalid_clone_atomics.iter().map(|t| t.get_id(Some(context.interner))).collect::<Vec<_>>().join("|");

            if has_cloneable_object || has_mixed_type {
                context.collector.report_with_code(
                    Code::POSSIBLY_INVALID_CLONE,
                    Issue::warning(format!(
                        "Expression of type `{}` might not be a cloneable object.",
                        object_type.get_id(Some(context.interner))
                    ))
                    .with_annotation(Annotation::primary(self.object.span()).with_message(format!(
                        "This could be of type `{invalid_types_str}`, which cannot be cloned"
                    )))
                    .with_note("Attempting to `clone` a non-object or an enum will result in a fatal error.")
                    .with_help("Ensure the value is a cloneable object before this operation, for example by using an `instanceof` check."),
                );
            } else {
                let (primary_message, note) = if invalid_clone_atomics.iter().any(|t| t.is_enum()) {
                    (
                        "This expression is an enum, which is not cloneable",
                        "PHP enums are singleton-like objects and cannot be cloned. This will cause a fatal `Error`.",
                    )
                } else {
                    (
                        "This expression is not an object",
                        "The `clone` operator can only be used on objects. This will cause a fatal `Error`.",
                    )
                };

                context.collector.report_with_code(
                    Code::INVALID_CLONE,
                    Issue::error(format!(
                        "Invalid `clone` operation on non-cloneable type `{invalid_types_str}`."
                    ))
                    .with_annotation(Annotation::primary(self.object.span()).with_message(primary_message))
                    .with_note(note)
                    .with_help("Remove the `clone` operator or ensure the expression evaluates to a cloneable object instance."),
                );
            }
        }

        let resulting_type = if !invalid_clone_atomics.is_empty() {
            Rc::new(if has_mixed_type {
                TUnion::new(vec![TAtomic::Mixed(TMixed::new()), TAtomic::Never])
            } else if has_cloneable_object {
                combine_union_types(&object_type, &get_never(), context.codebase, context.interner, false)
            } else {
                get_never()
            })
        } else {
            object_type
        };

        artifacts.set_rc_expression_type(self, resulting_type);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::Code;
    use crate::test_analysis;

    test_analysis! {
        name = clone_valid,
        code = indoc! {r#"
            <?php

            class Example {}

            function get_clone(Example $example): Example {
                return clone $example;
            }
        "#}
    }

    test_analysis! {
        name = clone_maybe_invalid_valid,
        code = indoc! {r#"
            <?php

            class Example {}

            function get_clone(Example|string $example): Example {
                return clone $example;
            }
        "#},
        issues = [
            Code::POSSIBLY_INVALID_CLONE,
            Code::INVALID_RETURN_STATEMENT,
        ]
    }

    test_analysis! {
        name = clone_enum,
        code = indoc! {r#"
            <?php

            enum Color { case Red; }

            function get_clone(Color $color): Color {
                return clone $color;
            }
        "#},
        issues = [
            Code::INVALID_CLONE,
            Code::NEVER_RETURN,
        ]
    }
}
