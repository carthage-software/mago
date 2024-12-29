use indoc::indoc;

use mago_formatter::settings::FormatSettings;

use crate::test_format;

#[test]
pub fn test_single_quote() {
    let code = indoc! {r#"
        <?php

        $a = "Hello, world!";
        $b = 'Hello, world!';
        $c = "Hello, 'world'!";
        $d = 'Hello, "world"!';
        $e = "Hello, \"world\"!";
        $f = 'Hello, \'world\'!';
        $g = "Hello, 'world'!";
        $h = 'Hello, \"world\"!';
    "#};

    let expected = indoc! {r#"
        <?php

        $a = 'Hello, world!';
        $b = 'Hello, world!';
        $c = "Hello, 'world'!";
        $d = 'Hello, "world"!';
        $e = "Hello, \"world\"!";
        $f = 'Hello, \'world\'!';
        $g = "Hello, 'world'!";
        $h = 'Hello, \"world\"!';
    "#};

    test_format(code, expected, FormatSettings { single_quote: true, ..Default::default() })
}

#[test]
pub fn test_double_quote() {
    let code = indoc! {r#"
        <?php

        $a = "Hello, world!";
        $b = 'Hello, world!';
        $c = "Hello, 'world'!";
        $d = 'Hello, "world"!';
        $e = "Hello, \"world\"!";
        $f = 'Hello, \'world\'!';
        $g = "Hello, 'world'!";
        $h = 'Hello, \"world\"!';
    "#};

    let expected = indoc! {r#"
        <?php

        $a = "Hello, world!";
        $b = "Hello, world!";
        $c = "Hello, 'world'!";
        $d = 'Hello, "world"!';
        $e = "Hello, \"world\"!";
        $f = 'Hello, \'world\'!';
        $g = "Hello, 'world'!";
        $h = 'Hello, \"world\"!';
    "#};

    test_format(code, expected, FormatSettings { single_quote: false, ..Default::default() })
}
