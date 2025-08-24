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
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::unary::cast_type_to_string;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for CompositeString<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut non_empty = false;
        let mut all_literals = true;
        let mut resulting_string = Some(String::new());
        let mut impossible = false;

        for part in self.parts().as_slice() {
            let part_type = match part {
                StringPart::Literal(literal_string_part) => {
                    non_empty = non_empty || !literal_string_part.value.is_empty();
                    if let Some(resulting_string) = resulting_string.as_mut() {
                        resulting_string.push_str(literal_string_part.value);
                    }

                    continue;
                }
                StringPart::Expression(expression) => {
                    let was_inside_general_use = block_context.inside_general_use;
                    block_context.inside_general_use = true;
                    expression.analyze(context, block_context, artifacts)?;
                    block_context.inside_general_use = was_inside_general_use;

                    artifacts.get_rc_expression_type(expression).cloned()
                }
                StringPart::BracedExpression(braced_expression) => {
                    let was_inside_general_use = block_context.inside_general_use;
                    block_context.inside_general_use = true;
                    braced_expression.expression.analyze(context, block_context, artifacts)?;
                    block_context.inside_general_use = was_inside_general_use;

                    artifacts.get_rc_expression_type(&braced_expression.expression).cloned()
                }
            };

            let Some(part_type) = part_type else {
                all_literals = false;
                resulting_string = None;

                // TODO: maybe it is worth reporting an issue here?
                continue;
            };

            let casted_part_type = cast_type_to_string(&part_type, context, block_context, artifacts, part.span())?;
            if casted_part_type.is_never() {
                impossible = true;

                continue;
            }

            let mut is_non_empty_part = true;
            let mut part_is_all_literals = true;
            let mut part_literal_string: Option<&str> = None;
            let mut could_specify_literals = true;

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
                        resulting_string = None;
                        part_literal_string = None;
                        could_specify_literals = false;
                    }
                    TStringLiteral::Value(literal_string) => {
                        if !could_specify_literals {
                            // We can't specify literals for this part, as
                            // one or more parts contain specified literal strings
                            // that are not identical.
                            continue;
                        }

                        match part_literal_string {
                            None => {
                                part_literal_string = Some(literal_string);
                            }
                            Some(previous_part_string) if !previous_part_string.eq(literal_string) => {
                                // This part of the string type contain a specified literal string
                                // that is different from a previous part.
                                part_literal_string = None;
                                could_specify_literals = false;
                            }
                            _ => {
                                // This part literal is equal to the previous one
                            }
                        }
                    }
                }
            }

            non_empty = non_empty || is_non_empty_part;
            all_literals = all_literals && part_is_all_literals;

            if !part_is_all_literals {
                resulting_string = None;
            } else if could_specify_literals
                && let Some(part_literal_string) = part_literal_string
                && let Some(resulting_string) = resulting_string.as_mut()
            {
                resulting_string.push_str(part_literal_string);
            }
        }

        let resulting_type = if impossible {
            get_never()
        } else if let Some(literal_string) = resulting_string {
            get_literal_string(atom(literal_string.as_ref()))
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
}
