use fennec_ast::ast::*;
use fennec_ast::Program;
use fennec_reporting::*;
use fennec_span::*;
use fennec_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

const STRICT_TYPES_DIRECTIVE: &str = "strict_types";

#[derive(Clone, Debug)]
pub struct RequireStrictTypesRule;

impl Rule for RequireStrictTypesRule {
    #[inline]
    fn get_name(&self) -> &'static str {
        "require-strict-types"
    }

    #[inline]
    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Warning)
    }
}

impl<'a> Walker<LintContext<'a>> for RequireStrictTypesRule {
    fn walk_program<'ast>(&self, program: &'ast Program, context: &mut LintContext<'a>) {
        if program.statements.len() < 2 {
            return;
        }

        let mut found = false;

        for statement in program.statements.iter() {
            if let Statement::Declare(declare) = statement {
                for item in declare.items.iter() {
                    let name = context.lookup(&item.name.value);

                    if name != STRICT_TYPES_DIRECTIVE {
                        continue;
                    }

                    match &item.value {
                        Expression::Literal(Literal::Integer(integer)) => {
                            let disabled = match &integer.value {
                                Some(val) => *val == 0,
                                None => {
                                    // ignore invalid values, as they will be caught by the semantics checker

                                    continue;
                                }
                            };

                            if disabled && !context.option("allow-disabling").and_then(|o| o.as_bool()).unwrap_or(false)
                            {
                                context.report(
                                    Issue::new(context.level(), "`strict_types` is disabled")
                                        .with_annotation(Annotation::primary(item.value.span()))
                                        .with_annotation(
                                            Annotation::secondary(item.name.span())
                                                .with_message("`strict_types` directive here"),
                                        )
                                        .with_note("disabling `strict_types` can lead to type safety issues.")
                                        .with_help("consider setting `strict_types` to `1` to enforce strict typing."),
                                );
                            }
                        }
                        _ => {
                            // ignore other values, as they will be caught by the semantics checker
                        }
                    };

                    found = true;
                }
            }
        }

        if !found {
            context.report(
                Issue::new(
                    context.level(),
                    "missing `declare(strict_types=1);` statement at the beginning of the file",
                )
                .with_annotation(Annotation::primary(program.span()))
                .with_note("`strict_types` enforces strict type checking, which can prevent subtle bugs.")
                .with_help("add `declare(strict_types=1);` at the top of your file."),
            );
        }
    }
}
