use indoc::indoc;

use mago_formatter::settings::FormatSettings;

use crate::test_format;

#[test]
pub fn test_dangling_block_comments() {
    let code = indoc! {r#"
        <?php

        class Foo
        {
            public function bar(): void
            {
                // This is a comment
            }
        }
    "#};

    let expected = indoc! {r#"
        <?php

        class Foo
        {
            public function bar(): void
            {
                // This is a comment
            }
        }
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_opening_tag_trailing_comments() {
    let code = indoc! {r#"
        <?php // some comment

        echo 'Hello, world!';
    "#};

    let expected = indoc! {r#"
        <?php // some comment

        echo 'Hello, world!';
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_opening_tag_trailing_comments_no_new_line() {
    let code = indoc! {r#"
        <?php // some comment
        echo 'Hello, world!';
    "#};

    let expected = indoc! {r#"
        <?php // some comment
        echo 'Hello, world!';
    "#};

    test_format(code, expected, FormatSettings::default())
}
