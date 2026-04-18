use mago_pattern::test_matches;
use mago_pattern::test_matches_with_captures;
use mago_pattern::test_no_match;

mod class_members {
    use super::*;

    test_matches! {
        name = class_property_plain,
        pattern = "public string $name;",
        code = "<?php class Foo { public string $name; }",
        expect = 1,
    }

    test_matches! {
        name = class_property_with_metavar_name,
        pattern = "public string $^name;",
        code = "<?php class Foo { public string $title; public string $subtitle; }",
        expect = 2,
    }

    test_matches_with_captures! {
        name = class_property_captures_name,
        pattern = "public string $^name;",
        code = "<?php class Foo { public string $title; }",
        expect = 1,
        captures = [("name", "$title")],
    }

    test_matches! {
        name = class_readonly_property,
        pattern = "public readonly string $^name;",
        code = "<?php class Foo { public readonly string $id; public string $mutable; }",
        expect = 1,
    }

    test_no_match! {
        name = class_readonly_property_wrong_order,
        pattern = "public readonly string $^name;",
        code = "<?php class Foo { private readonly string $id; }",
    }

    test_matches! {
        name = class_constant_literal_string,
        pattern = "public const string ^NAME = 'foo';",
        code = "<?php class Foo { public const string KEY = 'foo'; }",
        expect = 1,
    }
}

mod statements {
    use super::*;

    test_matches! {
        name = echo_statement,
        pattern = "echo ^msg;",
        code = "<?php echo $greeting; echo 'hi';",
        expect = 2,
    }

    test_matches! {
        name = return_statement,
        pattern = "return ^v;",
        code = "<?php function f() { return $x; } function g() { return 42; }",
        expect = 2,
    }

    test_matches! {
        name = global_statement,
        pattern = "global $^x;",
        code = "<?php function f() { global $config; }",
        expect = 1,
    }

    test_matches! {
        name = throw_statement,
        pattern = "throw new ^exc(^msg);",
        code = "<?php function f() { throw new RuntimeException('boom'); }",
        expect = 1,
    }
}

mod functions {
    use super::*;

    test_matches! {
        name = function_with_empty_body,
        pattern = "function ^name(): void {}",
        code = "<?php function noop(): void {}",
        expect = 1,
    }
}
