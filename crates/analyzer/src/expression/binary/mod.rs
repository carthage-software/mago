use std::rc::Rc;

use mago_codex::ttype::get_bool;
use mago_syntax::ast::Binary;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Expression;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::get_expression_range;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

pub mod utils;

mod arithmetic;
mod comparison;
mod concat;
mod logical;
mod null_coalesce;
mod spaceship;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Binary<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match &self.operator {
            BinaryOperator::Addition(_)
            | BinaryOperator::Subtraction(_)
            | BinaryOperator::Multiplication(_)
            | BinaryOperator::Division(_)
            | BinaryOperator::Modulo(_)
            | BinaryOperator::Exponentiation(_)
            | BinaryOperator::BitwiseAnd(_)
            | BinaryOperator::BitwiseOr(_)
            | BinaryOperator::BitwiseXor(_)
            | BinaryOperator::LeftShift(_)
            | BinaryOperator::RightShift(_) => {
                arithmetic::analyze_arithmetic_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::And(_) | BinaryOperator::LowAnd(_) => {
                logical::analyze_logical_and_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::Or(_) | BinaryOperator::LowOr(_) => {
                logical::analyze_logical_or_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::LowXor(_) => {
                logical::analyze_logical_xor_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::StringConcat(_) => {
                concat::analyze_string_concat_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::NullCoalesce(_) => {
                null_coalesce::analyze_null_coalesce_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::Spaceship(_) => {
                spaceship::analyze_spaceship_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::Equal(_)
            | BinaryOperator::NotEqual(_)
            | BinaryOperator::Identical(_)
            | BinaryOperator::NotIdentical(_)
            | BinaryOperator::AngledNotEqual(_)
            | BinaryOperator::LessThan(_)
            | BinaryOperator::LessThanOrEqual(_)
            | BinaryOperator::GreaterThan(_)
            | BinaryOperator::GreaterThanOrEqual(_) => {
                comparison::analyze_comparison_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::Instanceof(_) => {
                self.lhs.analyze(context, block_context, artifacts)?;

                if !matches!(
                    self.rhs,
                    Expression::Identifier(_) | Expression::Self_(_) | Expression::Static(_) | Expression::Parent(_)
                ) {
                    self.rhs.analyze(context, block_context, artifacts)?;
                }

                artifacts.expression_types.insert(get_expression_range(self), Rc::new(get_bool()));

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = concat_operator_test,
        code = indoc! {r#"
            <?php

            $name = "world";

            echo "Hello " . $name;
        "#}
    }

    test_analysis! {
        name = assertions_are_applied,
        code = indoc! {r"
            <?php

            const PHP_INT_MAX = 9223372036854775807;

            /**
             * @param int<1, max> $length
             * @return list<string>
             */
            function str_split(string $string, int $length = 1): array
            {
                return str_split($string, $length);
            }

            function intdiv(int $num1, int $num2): int
            {
                return intdiv($num1, $num2);
            }

            /**
             * @param string $character
             * @return int<0, 255>
             */
            function ord(string $character): int
            {
                return ord($character);
            }

            /**
             * @param non-empty-string $number
             * @param int<2, 36> $from_base
             */
            function from_base(string $number, int $from_base): int
            {
                $limit = intdiv(PHP_INT_MAX, $from_base);
                $result = 0;
                foreach (str_split($number, 1) as $digit) {
                    $oval = ord($digit);

                    if (/* '0' - '9' */ $oval >= 48 && $oval <= 57) {
                        $dval = $oval - 48;
                    } elseif (/* 'a' - 'z' */ $oval >= 97 && $oval <= 122) {
                        $dval = $oval - 87;
                    } elseif (/* 'A' - 'Z' */ $oval >= 65 && $oval <= 90) {
                        $dval = $oval - 55;
                    } else {
                        $dval = 99;
                    }

                    if ($from_base < $dval) {
                        exit('Invalid digit ' . $digit . ' in base ' . $from_base);
                    }

                    $oldval = $result;
                    $result = ($from_base * $result) + $dval;
                    if ($oldval > $limit || $oldval > $result) {
                        exit('Unexpected integer overflow parsing ' . $number . ' from base ' . $from_base);
                    }
                }

                return $result;
            }
        "}
    }

    test_analysis! {
        name = array_to_string_conversion_within_concat_operand,
        code = indoc! {r#"
            <?php

            $name = ["world"];

            echo "Hello " . $name;
        "#},
        issues = [
            IssueCode::ArrayToStringConversion,
        ]
    }

    test_analysis! {
        name = bitwise_or_binary_operator,
        code = indoc! {r"
            <?php

            const JSON_BIGINT_AS_STRING = 2;

            function x(): int
            {
                $a = JSON_BIGINT_AS_STRING | 1;

                return $a;
            }
        "},
    }

    test_analysis! {
        name = arithmetic_on_generics,
        code = indoc! {r"
            <?php

            /**
             * @template T of int|float
             *
             * @param T $start
             * @param T $end
             * @param T|null $step
             *
             * @return non-empty-list<T>
             */
            function range(int|float $start, int|float $end, int|float|null $step = null): array
            {
                if (((float) $start) === ((float) $end)) {
                    return [$start];
                }

                if ($start < $end) {
                    if (null === $step) {
                        $step = 1;
                    }

                    if ($step < 0) {
                        exit('If $end is greater than $start, then $step must be positive or null.');
                    }

                    $result = [];
                    for ($i = $start; $i <= $end; $i += $step) {
                        $result[] = $i;
                    }

                    return $result;
                }

                if (null === $step) {
                    $step = -1;
                }

                if ($step > 0) {
                    exit('If $start is greater than $end, then $step must be negative or null.');
                }

                $result = [];
                for ($i = $start; $i >= $end; $i += $step) {
                    $result[] = $i;
                }

                return $result;
            }
        "},
    }

    test_analysis! {
        name = codepoints,
        code = indoc! {r"
            <?php

            // stub
            function chr(int $code): string {
                return (string) $code;
            }

            function from_code_points(int ...$code_points): string
            {
                $string = '';
                foreach ($code_points as $code) {
                    $code %= 0x200000;
                    if (0x80 > $code) {
                        $string .= chr($code);
                        continue;
                    }

                    if (0x800 > $code) {
                        $string .= chr(0xC0 | ($code >> 6)) . chr(0x80 | ($code & 0x3F));
                        continue;
                    }

                    if (0x10000 > $code) {
                        $string .= chr(0xE0 | ($code >> 12)) . chr(0x80 | (($code >> 6) & 0x3F));
                        $string .= chr(0x80 | ($code & 0x3F));
                        continue;
                    }

                    $string .= chr(0xF0 | ($code >> 18)) . chr(0x80 | (($code >> 12) & 0x3F));
                    $string .= chr(0x80 | (($code >> 6) & 0x3F)) . chr(0x80 | ($code & 0x3F));
                }

                return $string;
            }
        "},
    }

    test_analysis! {
        name = null_coalescing_mixed,
        code = indoc! {r"
            <?php

            function test($foo = null) {
                return $foo ?? 'bar';
            }
        "},
    }

    test_analysis! {
        name = cant_determine_if_types_are_identical_for_mixed_template,
        code = indoc! {r"
            <?php

            /**
             * @template T
             * @param T $x
             */
            function x(mixed $x): void {
                if (false === $x) {
                    echo 'X is false';
                } else {
                    echo 'X is not false';
                }
            }
        "},
    }

    test_analysis! {
        name = int_mod,
        code = indoc! {r"
            <?php

            const NANOSECONDS_PER_SECOND = 1_000_000_000;

            const MICROSECONDS_PER_SECOND = 1_000_000;

            const MILLISECONDS_PER_SECOND = 1000;

            const SECONDS_PER_MINUTE = 60;

            const SECONDS_PER_HOUR = 3600;

            final readonly class Duration
            {
                /**
                 * @param int $hours
                 * @param int<-59, 59> $minutes
                 * @param int<-59, 59> $seconds
                 * @param int<-999999999, 999999999> $nanoseconds
                 *
                 * @pure
                 */
                private function __construct(
                    private int $hours,
                    private int $minutes,
                    private int $seconds,
                    private int $nanoseconds,
                ) {}

                public function getTotalNanoseconds(): int {
                    return ($this->hours * SECONDS_PER_HOUR * NANOSECONDS_PER_SECOND)
                         + ($this->minutes * SECONDS_PER_MINUTE * NANOSECONDS_PER_SECOND)
                         + ($this->seconds * NANOSECONDS_PER_SECOND)
                         + $this->nanoseconds;
                }

                /**
                 * @pure
                 */
                public static function fromParts(int $hours, int $minutes = 0, int $seconds = 0, int $nanoseconds = 0): self
                {
                    $s =
                        (SECONDS_PER_HOUR * $hours) +
                        (SECONDS_PER_MINUTE * $minutes) +
                        $seconds +
                        ((int) ($nanoseconds / NANOSECONDS_PER_SECOND));

                    $ns = $nanoseconds % NANOSECONDS_PER_SECOND;

                    if ($s < 0 && $ns > 0) {
                        ++$s;
                        $ns -= NANOSECONDS_PER_SECOND;
                    } elseif ($s > 0 && $ns < 0) {
                        --$s;
                        $ns += NANOSECONDS_PER_SECOND;
                    }

                    $m = (int) ($s / 60);
                    $s %= 60;
                    $h = (int) ($m / 60);
                    $m %= 60;

                    return new self($h, $m, $s, $ns);
                }
            }
        "},
    }

    test_analysis! {
        name = string_manipulation,
        code = indoc! {r#"
            <?php

            const STR_PAD_RIGHT = 0;

            const STR_PAD_LEFT = 1;

            /**
             * @pure
             */
            function abs(int|float $num): int|float
            {
                return abs($num);
            }

            /**
             * @pure
             */
            function str_pad(string $string, int $length, string $pad_string = ' ', int $pad_type = STR_PAD_RIGHT): string
            {
                return str_pad($string, $length, $pad_string, $pad_type);
            }

            /**
             * @pure
             */
            function substr(string $string, int $offset, null|int $length = null): string
            {
                return substr($string, $offset, $length);
            }

            /**
             * @pure
             */
            function rtrim(string $string, string $characters = " \n\r\t\v\0"): string
            {
                return rtrim($string, $characters);
            }

            /**
             * @param array<string>|string $separator
             * @param array<string>|null $array
             *
             * @pure
             */
            function join(array|string $separator = '', null|array $array = null): string
            {
                return join($separator, $array);
            }

            /**
             * @param int $hours
             * @param int<-59, 59> $minutes
             * @param int<-59, 59> $seconds
             * @param int<-999999999, 999999999> $nanoseconds
             * @param int<0, max> $max_decimals
             *
             * @pure
             */
            function format_duration(int $hours, int $minutes, int $seconds, int $nanoseconds, int $max_decimals = 3): string
            {
                $decimal_part = '';
                if ($max_decimals > 0) {
                    $decimal_part = (string) abs($nanoseconds);
                    $decimal_part = str_pad($decimal_part, 9, '0', STR_PAD_LEFT);
                    $decimal_part = substr($decimal_part, 0, $max_decimals);
                    $decimal_part = rtrim($decimal_part, '0');
                }

                if ($decimal_part !== '') {
                    $decimal_part = '.' . $decimal_part;
                }

                $sec_sign = $seconds < 0 || $nanoseconds < 0 ? '-' : '';
                $sec = abs($seconds);

                $containsHours = $hours !== 0;
                $containsMinutes = $minutes !== 0;
                $concatenatedSeconds = $sec_sign . ((string) $sec) . $decimal_part;
                $containsSeconds = $concatenatedSeconds !== '0';

                /** @var list<non-empty-string> $output */
                $output = [];
                if ($containsHours) {
                    $output[] = ((string) $hours) . ' hour(s)';
                }

                if ($containsMinutes || $containsHours && $containsSeconds) {
                    $output[] = ((string) $minutes) . ' minute(s)';
                }

                if ($containsSeconds) {
                    $output[] = $concatenatedSeconds . ' second(s)';
                }

                return [] === $output ? '0 second(s)' : join(', ', $output);
            }
        "#},
    }

    test_analysis! {
        name = assert_instanceof_class_string,
        code = indoc! {r"
            <?php

            /**
             * @template T as object
             */
            final readonly class InstanceOfType
            {
                /**
                 * @var class-string<T> $classname
                 */
                private string $classname;

                /**
                 * @psalm-mutation-free
                 *
                 * @param class-string<T> $classname
                 */
                public function __construct(string $classname)
                {
                    $this->classname = $classname;
                }

                /**
                 * @psalm-assert-if-true T $value
                 */
                public function matches(mixed $value): bool
                {
                    return $value instanceof $this->classname;
                }

                /**
                 * @return T
                 */
                public function coerce(mixed $value): object
                {
                    if ($value instanceof $this->classname) {
                        return $value;
                    }

                    return $this->assert($value);
                }

                /**
                 * @return T
                 *
                 * @psalm-assert T $value
                 */
                public function assert(mixed $value): object
                {
                    if ($value instanceof $this->classname) {
                        return $value;
                    }

                    return $this->coerce($value);
                }

                public function toString(): string
                {
                    return $this->classname;
                }
            }
        "},
    }
}
