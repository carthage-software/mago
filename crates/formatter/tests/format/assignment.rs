use indoc::indoc;

use mago_formatter::settings::FormatSettings;

use crate::test_format;

#[test]
pub fn test_all_nodes() {
    let code = indoc! {r#"
        <?php

        $a = 1;

        class A {
            public const CONSTANT = 2;
        }

        const GLOBAL_CONSTANT = 3;

        enum Status: int {
            case Active = 1;
        }

        class B {
            public int $property = 4;
        }

        $arr = [
            'key1' => 'value1',
            'key2' => 'value2'
        ];
    "#};

    let expected = indoc! {r#"
        <?php

        $a = 1;

        class A
        {
            public const CONSTANT = 2;
        }

        const GLOBAL_CONSTANT = 3;

        enum Status: int
        {
            case Active = 1;
        }

        class B
        {
            public int $property = 4;
        }

        $arr = [
            'key1' => 'value1',
            'key2' => 'value2',
        ];
    "#};

    // Test formatting for all `AssignmentLikeNode` types
    test_format(code, expected, FormatSettings::default());
}

#[test]
pub fn test_chain() {
    let code = indoc! {r#"
        <?php

        $a = $b = $c = 1;
        $x =
        $y =
        $z =
        [
            5,
            6,
            7 => function () {
                return 8;
            }
        ]
        ;
    "#};

    let expected = indoc! {r#"
        <?php

        $a = $b = $c = 1;
        $x =
            $y =
                $z = [
                    5,
                    6,
                    7 => function () {
                        return 8;
                    },
                ];
    "#};

    test_format(code, expected, FormatSettings::default());
}

#[test]
pub fn test_break_after_operator() {
    let code = indoc! {r#"
        <?php

        $long_variable_name = $another_long_variable_name;
    "#};

    let expected_wide = indoc! {r#"
        <?php

        $long_variable_name = $another_long_variable_name;
    "#};

    test_format(code, expected_wide, FormatSettings { print_width: 120, ..Default::default() });

    let expected_narrow = indoc! {r#"
        <?php

        $long_variable_name =
            $another_long_variable_name;
    "#};

    test_format(code, expected_narrow, FormatSettings { print_width: 40, ..Default::default() });
}

#[test]
pub fn test_conditional_assignment() {
    let code = indoc! {r#"
        <?php

        $response_body = (false !== $responseBody && '' !== $responseBody && 'GET' !== $request->getMethod())
                                ? $responseBody : null;
    "#};

    let expected_super_wide = indoc! {r#"
        <?php

        $response_body = false !== $responseBody && '' !== $responseBody && 'GET' !== $request->getMethod() ? $responseBody : null;
    "#};

    test_format(code, expected_super_wide, FormatSettings { print_width: 200, ..Default::default() });

    let expected_wide = indoc! {r#"
        <?php

        $response_body =
            false !== $responseBody && '' !== $responseBody && 'GET' !== $request->getMethod() ? $responseBody : null;
    "#};

    test_format(code, expected_wide, FormatSettings { print_width: 120, ..Default::default() });

    let expected_narrow = indoc! {r#"
        <?php

        $response_body =
            false !== $responseBody && '' !== $responseBody && 'GET' !== $request->getMethod()
                ? $responseBody
                : null;
    "#};

    test_format(code, expected_narrow, FormatSettings { print_width: 105, ..Default::default() });

    let expected_super_narrow = indoc! {r#"
        <?php

        $response_body =
            false !== $responseBody &&
            '' !== $responseBody &&
            'GET' !== $request->getMethod()
                ? $responseBody
                : null;
    "#};

    test_format(code, expected_super_narrow, FormatSettings { print_width: 40, ..Default::default() });
}
