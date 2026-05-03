use mago_atom::atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::atomic::scalar::string::TStringLiteral;
use mago_codex::ttype::get_literal_string;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_non_empty_unspecified_literal_string;
use mago_codex::ttype::get_string;
use mago_codex::ttype::get_unspecified_literal_string;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;
use mago_syntax::ast::CompositeString;
use mago_syntax::ast::StringPart;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::unary::cast_type_to_string;
use crate::utils::expression::get_expression_id;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for CompositeString<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut non_empty = false;
        let mut all_literals = true;
        let mut resulting_strings: Option<Vec<String>> = Some(vec![String::new()]);
        let mut impossible = false;

        for part in self.parts().as_slice() {
            let (part_type, part_expression_id) = match part {
                StringPart::Literal(literal_string_part) => {
                    non_empty = non_empty || !literal_string_part.value.is_empty();
                    if let Some(strings) = resulting_strings.as_mut() {
                        for s in strings.iter_mut() {
                            s.push_str(literal_string_part.value);
                        }
                    }

                    continue;
                }
                StringPart::Expression(expression) => {
                    let was_inside_general_use = block_context.flags.inside_general_use();
                    block_context.flags.set_inside_general_use(true);
                    expression.analyze(context, block_context, artifacts)?;
                    block_context.flags.set_inside_general_use(was_inside_general_use);

                    (
                        artifacts.get_rc_expression_type(expression).cloned(),
                        get_expression_id(
                            expression,
                            block_context.scope.get_class_like_name(),
                            context.resolved_names,
                            Some(context.codebase),
                        ),
                    )
                }
                StringPart::BracedExpression(braced_expression) => {
                    let was_inside_general_use = block_context.flags.inside_general_use();
                    block_context.flags.set_inside_general_use(true);
                    braced_expression.expression.analyze(context, block_context, artifacts)?;
                    block_context.flags.set_inside_general_use(was_inside_general_use);

                    (
                        artifacts.get_rc_expression_type(&braced_expression.expression).cloned(),
                        get_expression_id(
                            braced_expression.expression,
                            block_context.scope.get_class_like_name(),
                            context.resolved_names,
                            Some(context.codebase),
                        ),
                    )
                }
            };

            let Some(part_type) = part_type else {
                all_literals = false;
                resulting_strings = None;

                // TODO: maybe it is worth reporting an issue here?
                continue;
            };

            let casted_part_type = cast_type_to_string(
                &part_type,
                part_expression_id.as_deref(),
                context,
                block_context,
                artifacts,
                part.span(),
            )?;

            if casted_part_type.is_never() {
                impossible = true;

                continue;
            }

            let mut is_non_empty_part = true;
            let mut part_is_all_literals = true;
            let mut has_unspecified_literal = false;
            let mut part_literal_values: Vec<&str> = Vec::new();

            for cast_part_atomic in casted_part_type.types.as_ref() {
                is_non_empty_part = is_non_empty_part && cast_part_atomic.is_non_empty_string();

                if !part_is_all_literals {
                    continue;
                }

                let TAtomic::Scalar(TScalar::String(TString { literal: Some(literal), .. })) = cast_part_atomic else {
                    part_is_all_literals = false;

                    continue;
                };

                match literal {
                    TStringLiteral::Unspecified => {
                        has_unspecified_literal = true;
                    }
                    TStringLiteral::Value(literal_string) => {
                        if !part_literal_values.contains(&literal_string.as_str()) {
                            part_literal_values.push(literal_string);
                        }
                    }
                }
            }

            non_empty = non_empty || is_non_empty_part;
            all_literals = all_literals && part_is_all_literals;

            if !part_is_all_literals || has_unspecified_literal {
                resulting_strings = None;
            } else if let Some(strings) = resulting_strings.as_mut() {
                if part_literal_values.len() == 1 {
                    for s in strings.iter_mut() {
                        s.push_str(part_literal_values[0]);
                    }
                } else {
                    let mut new_strings = Vec::with_capacity(strings.len() * part_literal_values.len());
                    for s in strings.iter() {
                        for val in &part_literal_values {
                            let mut fork = s.clone();
                            fork.push_str(val);
                            new_strings.push(fork);
                        }
                    }

                    *strings = new_strings;
                }
            } else {
                // resulting_strings was already cleared by an earlier non-literal part; keep it as None
            }

            if resulting_strings
                .as_ref()
                .is_some_and(|s| s.len() > context.settings.string_combination_threshold as usize)
            {
                resulting_strings = None;
            }
        }

        let resulting_type = if impossible {
            get_never()
        } else if let Some(literal_strings) = resulting_strings {
            if literal_strings.len() == 1 {
                get_literal_string(atom(literal_strings[0].as_ref()))
            } else {
                TUnion::from_vec(
                    literal_strings
                        .iter()
                        .map(|s| TAtomic::Scalar(TScalar::literal_string(atom(s.as_ref()))))
                        .collect(),
                )
            }
        } else if non_empty {
            if all_literals { get_non_empty_unspecified_literal_string() } else { get_non_empty_string() }
        } else if all_literals {
            get_unspecified_literal_string()
        } else {
            get_string()
        };

        artifacts.set_expression_type(self, resulting_type);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = correctly_identifies_non_empty_string_from_expression,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-string $x
             * @return non-empty-string
             */
            function x(string $x): string
            {
                return "$x";
            }
        "#}
    }

    test_analysis! {
        name = correctly_identifies_literal_strings_from_expression,
        code = indoc! {r#"
            <?php

            /**
             * @param 'X' $x
             * @param 'Y' $y
             * @return 'Hello, X and Y!'
             */
            function hello(string $x, string $y): string
            {
                return "Hello, $x and $y!";
            }
        "#}
    }

    test_analysis! {
        name = composite_string_all_literal_parts_non_empty,
        code = indoc! {r#"
            <?php

            /** @return "Hello world!" */
            function get_greeting(): string {
                $name = "world";
                return "Hello $name!";
            }
        "#}
    }

    test_analysis! {
        name = composite_string_all_literal_parts_can_be_empty,
        code = indoc! {r#"
            <?php
            /**
             * @param ""|literal-string $name
             * @return literal-string
             */
            function get_greeting_optional_name(string $name): string {
                return "Hello $name";
            }

            /**
             * @param ""|"user" $name_part
             * @return non-empty-literal-string
             */
            function get_prefix_maybe(string $name_part, bool $flag): string {
                 $prefix = "";
                 if ($flag) {
                    $prefix = "prefix";
                }

                return "$prefix-$name_part";
            }

            /**
             * @param ""|"A" $p1
             * @param ""|"B" $p2
             * @return literal-string
             */
            function combine_optional_parts(string $p1, string $p2): string {
                return "$p1$p2";
            }
        "#}
    }

    test_analysis! {
        name = composite_string_part_type_unknown,
        code = indoc! {r#"
            <?php

            /** @return non-empty-string */
            function get_string_with_unknown_part(): string {
                return "Value: $undefinedVar";
            }
        "#},
        issues = [
            IssueCode::UndefinedVariable,
        ]
    }

    test_analysis! {
        name = composite_string_array_to_string,
        code = indoc! {r#"
            <?php

            /** @return 'Array: Array' */
            function get_string_with_array(): string {
                $arr = [1, 2];
                return "Array: $arr";
            }
        "#},
        issues = [
            IssueCode::ArrayToStringConversion,
        ]
    }

    test_analysis! {
        name = composite_string_object_no_to_string,
        code = indoc! {r#"
            <?php

            class MySimpleClass {}

            /** @return non-empty-string */
            function get_string_with_object(MySimpleClass $obj): string {
                return "Object: $obj";
            }
        "#},
        issues = [
            IssueCode::InvalidTypeCast,
        ]
    }

    test_analysis! {
        name = composite_string_null_interpolated,
        code = indoc! {r#"
            <?php

            /** @return 'Value: ' (literal) */
            function get_string_with_null(): string {
                $val = null;
                return "Value: $val";
            }
        "#},
    }

    test_analysis! {
        name = composite_string_bools_interpolated,
        code = indoc! {r#"
            <?php

            /** @return 'T:1 F:' (literal) */
            function get_string_with_bools(): string {
                $t = true;
                $f = false;

                return "T:$t F:$f";
            }
        "#},
    }

    test_analysis! {
        name = composite_string_all_empty_literals,
        code = indoc! {r#"
            <?php

            /** @return "" (literal) */
            function get_empty_string_from_parts(): string {
                $a = "";
                $b = "";
                return "$a$b";
            }
        "#}
    }

    test_analysis! {
        name = composite_string_literal_and_general_string_non_empty,
        code = indoc! {r#"
            <?php

            /**
             * @param string $name
             * @return non-empty-string
             */
            function greet(string $name): string {
                return "Hello $name!";
            }
        "#}
    }

    test_analysis! {
        name = composite_string_literal_and_non_empty_string,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-string $name
             * @return non-empty-string
             */
            function greet_strong(string $name): string {
                return "User: $name";
            }
        "#}
    }

    test_analysis! {
        name = composite_string_dynamic_could_be_empty,
        code = indoc! {r#"
            <?php
            /**
             * @param string $middlePart
             * @return string
             */
            function frame_string(string $middlePart): string {
                return "$middlePart";
            }
        "#},
    }

    test_analysis! {
        name = composite_string_literal_zero_string,
        code = indoc! {r#"
            <?php

            /** @return 'Count: 0' */
            function get_count_zero_string(): string {
                $countStr = "0";
                return "Count: $countStr";
            }
        "#},
    }

    test_analysis! {
        name = composite_string_int_interpolated,
        code = indoc! {r#"
            <?php

            /** @return 'Age: 25' */
            function describe_age(): string {
                $age = 25;
                return "Age: $age";
            }
        "#},
    }

    test_analysis! {
        name = composite_string_float_interpolated,
        code = indoc! {r#"
            <?php

            /** @return 'Price: 10.99' */
            function describe_price(): string {
                $price = 10.99;
                return "Price: $price";
            }
        "#},
    }

    test_analysis! {
        name = composite_string_all_unspecified_non_empty_literals,
        code = indoc! {r#"
            <?php

            /**
             * @param literal-string $s1
             * @param literal-string $s2
             * @return non-empty-literal-string
             */
            function combine_literals(string $s1, string $s2): string {
                return "$s1-$s2";
            }
        "#}
    }

    test_analysis! {
        name = composite_string_union_of_different_literals,
        code = indoc! {r#"
            <?php

            /**
             * @param 'red'|'v' $c
             * @return 'hist8_red'|'hist8_v'
             */
            function build_key(string $c): string {
                return "hist8_{$c}";
            }
        "#}
    }
}
