use indoc::indoc;

use mago_formatter::settings::FormatSettings;

use crate::test_format;

#[test]
pub fn test_use_basic_sorting() {
    let code = indoc! {r#"
        <?php

        use B;
        use A;
        use function D;
        use function C;
        use const F;
        use const E;
    "#};

    let expected = indoc! {r#"
        <?php

        use A;
        use B;
        use function C;
        use function D;
        use const E;
        use const F;
    "#};

    test_format(
        code,
        expected,
        FormatSettings { sort_uses: true, separate_use_types: false, ..FormatSettings::default() },
    );
}

#[test]
pub fn test_use_sorting_with_separation() {
    let code = indoc! {r#"
        <?php

        use B;
        use A;
        use function D;
        use function C;
        use const F;
        use const E;
    "#};

    let expected = indoc! {r#"
        <?php

        use A;
        use B;

        use function C;
        use function D;

        use const E;
        use const F;
    "#};

    test_format(
        code,
        expected,
        FormatSettings { sort_uses: true, separate_use_types: true, ..FormatSettings::default() },
    );
}

#[test]
pub fn test_use_sorting_with_expansion() {
    let code = indoc! {r#"
        <?php

        use Example\B;
        use Example\{A, C};
        use function Example\e;
        use function Example\{d, f};
        use const Example\H;
        use const Example\{G, I};
    "#};

    let expected = indoc! {r#"
        <?php

        use Example\A;
        use Example\B;
        use Example\C;
        use function Example\d;
        use function Example\e;
        use function Example\f;
        use const Example\G;
        use const Example\H;
        use const Example\I;
    "#};

    test_format(
        code,
        expected,
        FormatSettings {
            sort_uses: true,
            expand_use_groups: true,
            separate_use_types: false,
            ..FormatSettings::default()
        },
    );
}

#[test]
pub fn test_use_sorting_separation_expansion() {
    let code = indoc! {r#"
        <?php

        use B;
        use A;
        use Foo\{Bar, Baz};
        use function D;
        use function C;
        use function Qux\{Quuz\Corgi, Quux};
        use const F;
        use const E;
    "#};

    let expected = indoc! {r#"
        <?php

        use A;
        use B;
        use Foo\Bar;
        use Foo\Baz;

        use function C;
        use function D;
        use function Qux\Quux;
        use function Qux\Quuz\Corgi;

        use const E;
        use const F;
    "#};

    test_format(code, expected, FormatSettings::default());
}

#[test]
pub fn test_use_no_changes() {
    let code = indoc! {r#"
        <?php

        use B;
        use A;
        use function D;
        use function C;
        use const F;
        use const E;
        use Foo\{Bar, Baz};
    "#};

    let expected = indoc! {r#"
        <?php

        use B;
        use A;
        use function D;
        use function C;
        use const F;
        use const E;
        use Foo\{Bar, Baz};
    "#};

    test_format(
        code,
        expected,
        FormatSettings {
            expand_use_groups: false,
            sort_uses: false,
            separate_use_types: false,
            ..FormatSettings::default()
        },
    );
}

#[test]
pub fn test_use_mixed_use_list() {
    let code = indoc! {r#"
        <?php

        use MyNamespace\{
            const C,
            function F as funcF,
            A,
            B,
            function G,
            const D as constD,
        };
    "#};

    let expected = indoc! {r#"
        <?php

        use MyNamespace\{A, B, function F as funcF, function G, const C, const D as constD};
    "#};

    test_format(code, expected, FormatSettings { expand_use_groups: false, ..FormatSettings::default() });
}

#[test]
pub fn test_use_mixed_use_list_expanded() {
    let code = indoc! {r#"
        <?php

        use MyNamespace\{
            const C,
            function F as funcF,
            A,
            B,
            function G,
            const D as constD,
        };
    "#};

    let expected = indoc! {r#"
        <?php

        use MyNamespace\A;
        use MyNamespace\B;

        use function MyNamespace\F as funcF;
        use function MyNamespace\G;

        use const MyNamespace\C;
        use const MyNamespace\D as constD;
    "#};

    test_format(
        code,
        expected,
        FormatSettings {
            sort_uses: true,
            expand_use_groups: true,
            separate_use_types: true,
            ..FormatSettings::default()
        },
    );
}

#[test]
pub fn test_use_typed_use_list_expanded() {
    let code = indoc! {r#"
        <?php

        use function MyNamespace\{
            F as funcF,
            G,
        };
        use const MyNamespace2\{
            C,
            D as constD,
        };

    "#};

    let expected = indoc! {r#"
        <?php

        use function MyNamespace\F as funcF;
        use function MyNamespace\G;

        use const MyNamespace2\C;
        use const MyNamespace2\D as constD;
    "#};

    test_format(
        code,
        expected,
        FormatSettings {
            sort_uses: true,
            expand_use_groups: true,
            separate_use_types: true,
            ..FormatSettings::default()
        },
    );
}

#[test]
pub fn test_use_nested_namespaces_expanded() {
    let code = indoc! {r#"
        <?php

        use Foo\Bar\Baz;
        use Foo\Bar;
        use Foo;
    "#};

    let expected = indoc! {r#"
        <?php

        use Foo;
        use Foo\Bar;
        use Foo\Bar\Baz;
    "#};

    test_format(
        code,
        expected,
        FormatSettings { sort_uses: true, expand_use_groups: true, ..FormatSettings::default() },
    );
}

#[test]
pub fn test_adds_empty_line_after_use() {
    let code = indoc! {r#"
        <?php

        use Foo;
        class A extends Foo {}
    "#};

    let expected = indoc! {r#"
        <?php

        use Foo;

        class A extends Foo
        {
        }
    "#};

    test_format(code, expected, FormatSettings::default());
}

#[test]
pub fn test_leaves_single_empty_line_after_use() {
    let code = indoc! {r#"
        <?php

        use Foo;



        class A extends Foo
        {
        }
    "#};

    let expected = indoc! {r#"
        <?php

        use Foo;

        class A extends Foo
        {
        }
    "#};

    test_format(code, expected, FormatSettings::default());
}

#[test]
pub fn test_docs_before_use_are_preserved() {
    let code = indoc! {r#"
        <?php

        /**
         * This is a doc comment
         */

        use Foo; // This is a use statement

        class A extends Foo
        {
        }
    "#};

    let expected = indoc! {r#"
        <?php

        /**
         * This is a doc comment
         */

        use Foo; // This is a use statement

        class A extends Foo
        {
        }
    "#};

    test_format(code, expected, FormatSettings::default());
}
