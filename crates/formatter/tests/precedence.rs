mod runner {
    use std::borrow::Cow;

    use bumpalo::Bump;
    use mago_formatter::Formatter;
    use mago_formatter::settings::FormatSettings;
    use mago_php_version::PHPVersion;

    fn assert_expression_formatting(name: &'static str, formatted: &str, expected: &'static str, idempotency: bool) {
        let formatted = formatted.trim_start().trim_start_matches("<?=").trim_start();
        let formatted = formatted.trim_end().trim_end_matches(';').trim_end();

        if idempotency {
            assert_eq!(formatted, expected, "Expression `{name}` formatting is not idempotent");

            return;
        }

        assert_eq!(formatted, expected, "Expression `{name}` formatting does not match expected");
    }

    pub fn run_format_test(name: &'static str, input_expression: &'static str, expected_expression: &'static str) {
        let arena = Bump::new();
        let formatter = Formatter::new(
            &arena,
            PHPVersion::LATEST,
            FormatSettings {
                print_width: 512, // Large enough to avoid line breaks in tests
                ..FormatSettings::default()
            },
        );

        let code = "<?= ".to_string() + input_expression + ";";
        let formatted_code = formatter.format_code(Cow::Borrowed(name), Cow::Owned(code)).unwrap();
        assert_expression_formatting(name, formatted_code, expected_expression, false);

        let reformatted_code =
            formatter.format_code(Cow::Borrowed(name), Cow::Owned(formatted_code.to_owned())).unwrap();
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
        "$value = &$data[$field->getName()] ?? null",
        "($value = &$data[$field->getName()]) ?? null"
    );

    test_expression_format!(assign_ref_static_call, "$a = &B::c()", "$a = &B::c()");
    test_expression_format!(assign_ref_func_call, "$a = &b()", "$a = &b()");
    test_expression_format!(assign_ref_method_call, "$a = &$b->c()", "$a = &$b->c()");
    test_expression_format!(assign_ref_null_method_call, "$a = &$b?->c()", "$a = &$b?->c()");
    test_expression_format!(as_is, "$a * $b", "$a * $b");
    test_expression_format!(keep_parens_on_assignment_lhs_of_logical_word, "($a = $b) and $c", "($a = $b) and $c");
    test_expression_format!(remove_parens_for_logical_precedence_1, "($a || $b) xor $c", "$a || $b xor $c");
    test_expression_format!(remove_parens_for_logical_precedence_2, "$a and ($b || $c)", "$a and $b || $c");
    test_expression_format!(keep_parens_for_shift_vs_concat, "$a . ($b << $c)", "$a . ($b << $c)");
    test_expression_format!(keep_parens_for_shift_vs_addition, "$a << ($b + $c)", "$a << ($b + $c)");
    test_expression_format!(keep_parens_in_ternary_condition, "$a > ($b && $c) ? $d : $e", "$a > ($b && $c) ? $d : $e");
    test_expression_format!(keep_redundant_simple_arithmetic, "($a * $b) + $c", "($a * $b) + $c");
    test_expression_format!(keep_redundant_nested_arithmetic, "$a + (($b - $c) * $d)", "$a + (($b - $c) * $d)");
    test_expression_format!(remove_logical, "($a && $b) || $c", "$a && $b || $c");
    test_expression_format!(remove_comparison, "($a > $b) && ($c < $d)", "$a > $b && $c < $d");
    test_expression_format!(remove_left_associative, "($a - $b) - $c", "$a - $b - $c");
    test_expression_format!(remove_right_associative, "$a ** ($b ** $c)", "$a ** $b ** $c");
    test_expression_format!(remove_unary_higher_precedence, "(-$a) * $b", "-$a * $b");
    test_expression_format!(remove_pre_inc_higher_precedence, "(++$a) ** $b", "++$a ** $b");
    test_expression_format!(remove_unnecessary_wrapping, "($a + $b)", "$a + $b");
    test_expression_format!(remove_deeply_nested_wrapping, "((((($a || $b)))))", "$a || $b");
    test_expression_format!(keep_simple_arithmetic, "$a * ($b + $c)", "$a * ($b + $c)");
    test_expression_format!(keep_nested_arithmetic, "(($a + $b) * $c) / $d", "(($a + $b) * $c) / $d");
    test_expression_format!(keep_logical, "$a && ($b || $c)", "$a && ($b || $c)");
    test_expression_format!(keep_comparison, "$a > ($b && $c)", "$a > ($b && $c)");
    test_expression_format!(keep_left_associative_override, "$a - ($b - $c)", "$a - ($b - $c)");
    test_expression_format!(keep_right_associative_override, "($a ** $b) ** $c", "($a ** $b) ** $c");
    test_expression_format!(keep_unary_lower_precedence, "!($a && $b)", "!($a && $b)");
    test_expression_format!(keep_unary_minus_on_pow, "-($a ** $b)", "-$a ** $b");
    test_expression_format!(remove_instanceof, "($a instanceof B) + $c", "$a instanceof B + $c");
    test_expression_format!(keep_ternary_in_binary, "($a ? $b : $c) . $d", "($a ? $b : $c) . $d");
    test_expression_format!(
        complex_1_messy,
        "(($a = (((((++$b * ((((-$c)))))))) + ($d / ($e ** $f))) && ($g || $h)))",
        "$a = ((++$b * -$c) + ($d / ($e ** $f))) && ($g || $h)"
    );
    test_expression_format!(
        complex_2_messy,
        "($a = $b) and (($c || $d) xor ($e && $f))",
        "($a = $b) and ($c || $d xor $e && $f)"
    );
    test_expression_format!(
        complex_3_messy,
        "$a = ($b << ($c + ($d * $e))) >> ($f - $g)",
        "$a = ($b << ($c + ($d * $e))) >> ($f - $g)"
    );
    test_expression_format!(complex_4_messy, "$a = ((!$b) + ((~$c * --$d) / @$e))", "$a = !$b + ((~$c * --$d) / @$e)");
    test_expression_format!(
        complex_5_messy,
        "($a = (($b + ($c * $d)) <=> (($e / $f) - $g)))",
        "$a = ($b + ($c * $d)) <=> (($e / $f) - $g)"
    );
    test_expression_format!(
        complex_6_messy,
        "(($a = (((((($b))) + ($c * $d)) > $e) && ((((($f))) & $g) | ($h ^ $i)))) or (($j = (((($k ?? $l)))))))",
        "($a = ($b + ($c * $d)) > $e && ($f & $g) | ($h ^ $i)) or ($j = $k ?? $l)"
    );
    test_expression_format!(
        complex_7_messy,
        "$a = ($b + ($c - (($d * $e) / ($f % ($g ** $h)))))",
        "$a = $b + ($c - (($d * $e) / ($f % ($g ** $h))))"
    );
    test_expression_format!(complex_8_messy, "$a = ((($b ?? ($c ?? $d))) ? $e : $f)", "$a = $b ?? $c ?? $d ? $e : $f");
    test_expression_format!(
        complex_9_messy,
        "$a = ($b > ($c && $d < $e) ? $f : $g)",
        "$a = $b > ($c && $d < $e) ? $f : $g"
    );
    test_expression_format!(complex_10_messy, "$a = ($b . ($b << $c) . $d)", "$a = $b . ($b << $c) . $d");
    test_expression_format!(complex_11_messy, "($a = (- ($b ** $c)))", "$a = -$b ** $c");
    test_expression_format!(error_control_include, "$a = (@include $b) === $c", "$a = (@include $b) === $c");
    test_expression_format!(error_control_new, "$a = (@(new Foo($x))) === $c", "$a = @new Foo($x) === $c");
    test_expression_format!(nonassoc_identical_parens_left, "($a === 'b') === $c", "($a === 'b') === $c");
    test_expression_format!(nonassoc_less_than_parens_left, "($a < $b) < $c", "($a < $b) < $c");
    test_expression_format!(nonassoc_identical_parens_right, "$a === ($b === $c)", "$a === ($b === $c)");
}
