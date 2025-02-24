use indoc::indoc;

use mago_formatter::settings::FormatSettings;

use crate::test_format;

#[test]
pub fn test_parenthesis_around_closure() {
    let code = indoc! {r#"
        <?php

        $result = ($fib = function($n) use (&$fib) {
            return $n <= 1 ? $n : $fib($n - 1) + $fib($n - 2);
        })(10);
    "#};

    let expected = indoc! {r#"
        <?php

        $result = ($fib = function ($n) use (&$fib) {
            return $n <= 1 ? $n : ($fib($n - 1) + $fib($n - 2));
        })(10);
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_keyword_as_method_name() {
    let code = indoc! {r#"
        <?php

        $unit = $unitNode?->print($context);
    "#};

    let expected = indoc! {r#"
        <?php

        $unit = $unitNode?->print($context);
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_long_attribute() {
    let code = indoc! {r#"
        <?php

        #[Route('route/path', name: 'very_very_very_very_very_very_long_route_name', methods: ['GET'])]
        class Foo {}
    "#};

    let expected = indoc! {r#"
        <?php

        #[Route(
            'route/path',
            name: 'very_very_very_very_very_very_long_route_name',
            methods: ['GET'],
        )]
        class Foo
        {
        }
    "#};

    test_format(code, expected, FormatSettings { print_width: 80, ..FormatSettings::default() })
}

#[test]
pub fn test_parenthesis_around_construct() {
    let code = indoc! {r#"
        <?php

        (require_once __DIR__ . '/../bootstrap/app.php')->handleRequest(Request::capture());
    "#};

    let expected = indoc! {r#"
        <?php

        (require_once __DIR__ . '/../bootstrap/app.php')->handleRequest(Request::capture());
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_interpolated_strings_are_preserved() {
    let code = indoc! {r#"
        <?php

        "$foo[$index_var]";
        "${foo[$index_var]}";
        "{$foo[$index_var]}";
    "#};

    let expected = indoc! {r#"
        <?php

        "$foo[$index_var]";
        "${foo[$index_var]}";
        "{$foo[$index_var]}";
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_closure_creation_is_not_broken_like_argument_lists() {
    let code = indoc! {r#"
        <?php

        return $this->longMethodName(


                                $this->longPropertyName->evenLongerMethodNameNowCausesTheDotsToWrapWithComma(...),

                        );


        return $this->longMethodName(
            $this->longPropertyName->evenLongerMethodNameNowCausesTheDotsToWrapWithComma(1, 2),
                );
    "#};

    let expected = indoc! {r#"
        <?php

        return $this->longMethodName($this->longPropertyName->evenLongerMethodNameNowCausesTheDotsToWrapWithComma(...));

        return $this->longMethodName($this->longPropertyName->evenLongerMethodNameNowCausesTheDotsToWrapWithComma(
            1,
            2,
        ));
    "#};

    test_format(code, expected, FormatSettings { print_width: 40, ..FormatSettings::default() })
}

#[test]
pub fn test_parenthesis_are_preserved() {
    let code = indoc! {r#"
        <?php

        var_dump((1 ? 2 : 3) ?: 4);
    "#};

    let expected = indoc! {r#"
        <?php

        var_dump((1 ? 2 : 3) ?: 4);
    "#};

    test_format(code, expected, FormatSettings { print_width: 40, ..FormatSettings::default() })
}

#[test]
pub fn test_attributes_are_preserved() {
    let code = indoc! {r#"
        <?php

        #[ATTRIBUTE]
        function aaa(
          int $bbb,
          // this comment causes attribute to dissappear
          int $ccc
        ) {
          var_dump('test');
        }
    "#};

    let expected = indoc! {r#"
        <?php

        #[ATTRIBUTE]
        function aaa(
            int $bbb,
            // this comment causes attribute to dissappear
            int $ccc,
        ) {
            var_dump('test');
        }
    "#};

    test_format(code, expected, FormatSettings { print_width: 40, ..FormatSettings::default() })
}
