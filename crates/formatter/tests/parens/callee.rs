use indoc::indoc;

use mago_formatter::settings::FormatSettings;
use mago_php_version::PHPVersion;

use crate::test_format;
use crate::test_format_with_version;

#[test]
pub fn test_callee_needs_parens() {
    let code = indoc! {r#"
        <?php

        $a = (foo())->bar();
        $c = (foo())->bar;
        $d = (foo())::bar();
        $e = (foo())[0];
        $f = new (foo())();
    "#};

    let expected = indoc! {r#"
        <?php

        $a = foo()->bar();
        $c = foo()->bar;
        $d = foo()::bar();
        $e = foo()[0];
        $f = new (foo())();
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_instantiation_with_member_access_parentheses() {
    let code = indoc! {r#"
        <?php

        $a = (new Foo)->something();
        $a = (new Foo())->something();
        $a = (new Foo(1, 2))->something();
        $a = (new Foo(1, 2))->something()->else()->other()->thing();
    "#};

    let expected_php83 = indoc! {r#"
        <?php

        $a = (new Foo())->something();
        $a = (new Foo())->something();
        $a = (new Foo(1, 2))->something();
        $a = (new Foo(1, 2))
            ->something()
            ->else()
            ->other()
            ->thing();
    "#};

    let expected_php84 = indoc! {r#"
        <?php

        $a = new Foo()->something();
        $a = new Foo()->something();
        $a = new Foo(1, 2)->something();
        $a = new Foo(1, 2)
            ->something()
            ->else()
            ->other()
            ->thing();
    "#};

    let expected_php83_without_parens = indoc! {r#"
        <?php

        $a = (new Foo)->something();
        $a = (new Foo)->something();
        $a = (new Foo(1, 2))->something();
        $a = (new Foo(1, 2))
            ->something()
            ->else()
            ->other()
            ->thing();
    "#};

    let expected_php84_without_parens = indoc! {r#"
        <?php

        $a = (new Foo)->something();
        $a = (new Foo)->something();
        $a = new Foo(1, 2)->something();
        $a = new Foo(1, 2)
            ->something()
            ->else()
            ->other()
            ->thing();
    "#};

    test_format_with_version(code, expected_php83, PHPVersion::PHP83, FormatSettings::default());
    test_format_with_version(code, expected_php84, PHPVersion::PHP84, FormatSettings::default());
    test_format_with_version(
        code,
        expected_php83_without_parens,
        PHPVersion::PHP83,
        FormatSettings { parentheses_in_new_expression: false, ..FormatSettings::default() },
    );
    test_format_with_version(
        code,
        expected_php84_without_parens,
        PHPVersion::PHP84,
        FormatSettings { parentheses_in_new_expression: false, ..FormatSettings::default() },
    );
}
