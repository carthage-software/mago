use indoc::indoc;

use mago_formatter::settings::FormatSettings;

use crate::test_format;

pub mod assignment;
pub mod binaryish;
pub mod control_structure;
pub mod expression;
pub mod statement;
pub mod string;

#[test]
pub fn test_shebang() {
    let code = indoc! {r#"
        #!/usr/bin/env php
        <?php

        echo 'Hello, world!';
    "#};

    let expected = indoc! {r#"
        #!/usr/bin/env php
        <?php

        echo 'Hello, world!';
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_inline_html() {
    let code = indoc! {r#"
        <?php

        $a = 1;
        $b = 2;
        $c = $a + $b;

        ?>
        <a>a + b = <?= $c ?></a>
        <?php

        echo 'Hello, world!';
    "#};

    let expected = indoc! {r#"
        <?php

        $a = 1;
        $b = 2;
        $c = $a + $b;

        ?>
        <a>a + b = <?= $c ?></a>
        <?php

        echo 'Hello, world!';
    "#};

    test_format(code, expected, FormatSettings::default())
}

#[test]
pub fn test_php_85_constant_attributes() {
    let code = indoc! {r#"
        <?php

        #[Deprecated]
        const FOO = 'foo';
    "#};

    let expected = indoc! {r#"
        <?php

        #[Deprecated]
        const FOO = 'foo';
    "#};

    test_format(code, expected, FormatSettings::default())
}
