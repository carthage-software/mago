#![allow(clippy::unwrap_used, clippy::string_add)]

mod runner {
    use std::borrow::Cow;

    use mago_allocator::LocalArena;
    use mago_formatter::Formatter;
    use mago_formatter::settings::FormatSettings;
    use mago_php_version::PHPVersion;

    fn assert_expression_formatting(name: &'static str, formatted: &[u8], expected: &[u8], idempotency: bool) {
        let trimmed = formatted.trim_ascii_start();
        let trimmed = trimmed.strip_prefix(b"<?=").unwrap_or(trimmed).trim_ascii_start();
        let trimmed = trimmed.trim_ascii_end();
        let trimmed = mago_bytes::trim_end_byte(trimmed, b';').trim_ascii_end();

        if idempotency {
            assert_eq!(trimmed, expected, "Expression `{name}` formatting is not idempotent");

            return;
        }

        assert_eq!(trimmed, expected, "Expression `{name}` formatting does not match expected");
    }

    pub fn run_format_test(name: &'static str, input_expression: &'static [u8], expected_expression: &'static [u8]) {
        let arena = LocalArena::new();
        let formatter = Formatter::new(
            &arena,
            PHPVersion::LATEST,
            FormatSettings {
                print_width: 512, // Large enough to avoid line breaks in tests
                ..FormatSettings::default()
            },
        );

        let mut code: Vec<u8> = Vec::with_capacity(input_expression.len() + 4);
        code.extend_from_slice(b"<?=");
        code.extend_from_slice(input_expression);
        code.push(b';');
        let formatted_code = formatter.format_code(Cow::Borrowed(name.as_bytes()), Cow::Owned(code)).unwrap();
        assert_expression_formatting(name, formatted_code, expected_expression, false);

        let reformatted_code =
            formatter.format_code(Cow::Borrowed(name.as_bytes()), Cow::Owned(formatted_code.to_owned())).unwrap();
        assert_expression_formatting(name, reformatted_code, expected_expression, true);
    }
}

mod precedence {
    macro_rules! test_expression_format {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                crate::runner::run_format_test(stringify!($name), $input, $expected);
            }
        };
    }

    // The bug that started it all
    test_expression_format!(
        ben,
        b"$value = &$data[$field->getName()] ?? null",
        b"($value = &$data[$field->getName()]) ?? null"
    );

    test_expression_format!(assign_ref_static_call, b"$a = &B::c()", b"$a = &B::c()");
    test_expression_format!(assign_ref_func_call, b"$a = &b()", b"$a = &b()");
    test_expression_format!(assign_ref_method_call, b"$a = &$b->c()", b"$a = &$b->c()");
    test_expression_format!(assign_ref_null_method_call, b"$a = &$b?->c()", b"$a = &$b?->c()");
    test_expression_format!(as_is, b"$a * $b", b"$a * $b");
    test_expression_format!(keep_parens_on_assignment_lhs_of_logical_word, b"($a = $b) and $c", b"($a = $b) and $c");
    test_expression_format!(remove_parens_for_logical_precedence_1, b"($a || $b) xor $c", b"$a || $b xor $c");
    test_expression_format!(remove_parens_for_logical_precedence_2, b"$a and ($b || $c)", b"$a and $b || $c");
    test_expression_format!(keep_parens_for_shift_vs_concat, b"$a . ($b << $c)", b"$a . ($b << $c)");
    test_expression_format!(keep_parens_for_shift_vs_addition, b"$a << ($b + $c)", b"$a << ($b + $c)");
    test_expression_format!(
        keep_parens_in_ternary_condition,
        b"$a > ($b && $c) ? $d : $e",
        b"$a > ($b && $c) ? $d : $e"
    );
    test_expression_format!(keep_redundant_simple_arithmetic, b"($a * $b) + $c", b"($a * $b) + $c");
    test_expression_format!(keep_redundant_nested_arithmetic, b"$a + (($b - $c) * $d)", b"$a + (($b - $c) * $d)");
    test_expression_format!(remove_logical, b"($a && $b) || $c", b"$a && $b || $c");
    test_expression_format!(remove_comparison, b"($a > $b) && ($c < $d)", b"$a > $b && $c < $d");
    test_expression_format!(remove_left_associative, b"($a - $b) - $c", b"$a - $b - $c");
    test_expression_format!(remove_right_associative, b"$a ** ($b ** $c)", b"$a ** $b ** $c");
    test_expression_format!(remove_unary_higher_precedence, b"(-$a) * $b", b"-$a * $b");
    test_expression_format!(remove_pre_inc_higher_precedence, b"(++$a) ** $b", b"++$a ** $b");
    test_expression_format!(remove_unnecessary_wrapping, b"($a + $b)", b"$a + $b");
    test_expression_format!(remove_deeply_nested_wrapping, b"((((($a || $b)))))", b"$a || $b");
    test_expression_format!(keep_simple_arithmetic, b"$a * ($b + $c)", b"$a * ($b + $c)");
    test_expression_format!(keep_nested_arithmetic, b"(($a + $b) * $c) / $d", b"(($a + $b) * $c) / $d");
    test_expression_format!(keep_logical, b"$a && ($b || $c)", b"$a && ($b || $c)");
    test_expression_format!(keep_comparison, b"$a > ($b && $c)", b"$a > ($b && $c)");
    test_expression_format!(keep_left_associative_override, b"$a - ($b - $c)", b"$a - ($b - $c)");
    test_expression_format!(keep_right_associative_override, b"($a ** $b) ** $c", b"($a ** $b) ** $c");
    test_expression_format!(keep_unary_lower_precedence, b"!($a && $b)", b"!($a && $b)");
    test_expression_format!(keep_unary_minus_on_pow, b"-($a ** $b)", b"-$a ** $b");
    test_expression_format!(remove_instanceof, b"($a instanceof B) + $c", b"$a instanceof B + $c");
    test_expression_format!(keep_ternary_in_binary, b"($a ? $b : $c) . $d", b"($a ? $b : $c) . $d");
    test_expression_format!(
        complex_1_messy,
        b"(($a = (((((++$b * ((((-$c)))))))) + ($d / ($e ** $f))) && ($g || $h)))",
        b"$a = ((++$b * -$c) + ($d / ($e ** $f))) && ($g || $h)"
    );
    test_expression_format!(
        complex_2_messy,
        b"($a = $b) and (($c || $d) xor ($e && $f))",
        b"($a = $b) and ($c || $d xor $e && $f)"
    );
    test_expression_format!(
        complex_3_messy,
        b"$a = ($b << ($c + ($d * $e))) >> ($f - $g)",
        b"$a = ($b << ($c + ($d * $e))) >> ($f - $g)"
    );
    test_expression_format!(
        complex_4_messy,
        b"$a = ((!$b) + ((~$c * --$d) / @$e))",
        b"$a = !$b + ((~$c * --$d) / @$e)"
    );
    test_expression_format!(
        complex_5_messy,
        b"($a = (($b + ($c * $d)) <=> (($e / $f) - $g)))",
        b"$a = ($b + ($c * $d)) <=> (($e / $f) - $g)"
    );
    test_expression_format!(
        complex_6_messy,
        b"(($a = (((((($b))) + ($c * $d)) > $e) && ((((($f))) & $g) | ($h ^ $i)))) or (($j = (((($k ?? $l)))))))",
        b"($a = ($b + ($c * $d)) > $e && ($f & $g) | ($h ^ $i)) or ($j = $k ?? $l)"
    );
    test_expression_format!(
        complex_7_messy,
        b"$a = ($b + ($c - (($d * $e) / ($f % ($g ** $h)))))",
        b"$a = $b + ($c - (($d * $e) / ($f % ($g ** $h))))"
    );
    test_expression_format!(
        complex_8_messy,
        b"$a = ((($b ?? ($c ?? $d))) ? $e : $f)",
        b"$a = $b ?? $c ?? $d ? $e : $f"
    );
    test_expression_format!(
        complex_9_messy,
        b"$a = ($b > ($c && $d < $e) ? $f : $g)",
        b"$a = $b > ($c && $d < $e) ? $f : $g"
    );
    test_expression_format!(complex_10_messy, b"$a = ($b . ($b << $c) . $d)", b"$a = $b . ($b << $c) . $d");
    test_expression_format!(complex_11_messy, b"($a = (- ($b ** $c)))", b"$a = -$b ** $c");
    test_expression_format!(error_control_include, b"$a = (@include $b) === $c", b"$a = (@include $b) === $c");
    test_expression_format!(error_control_new, b"$a = (@(new Foo($x))) === $c", b"$a = @new Foo($x) === $c");
    test_expression_format!(nonassoc_identical_parens_left, b"($a === 'b') === $c", b"($a === 'b') === $c");
    test_expression_format!(nonassoc_less_than_parens_left, b"($a < $b) < $c", b"($a < $b) < $c");
    test_expression_format!(nonassoc_identical_parens_right, b"$a === ($b === $c)", b"$a === ($b === $c)");
}
