<?php

final class Issue2592Foo {}

class Issue2592Base {}

final class Issue2592Bar {}

/**
 * @param class-string $class
 */
function issue_2592_any_class_string(Issue2592Foo $foo, string $class): Issue2592Foo
{
    if ($foo instanceof $class) {
        return $foo;
    } else {
        return $foo;
    }
}

function issue_2592_plain_string(Issue2592Foo $foo, string $class): Issue2592Foo
{
    if (!$foo instanceof $class) {
        return $foo;
    }

    return $foo;
}

/**
 * @param class-string<Issue2592Base> $class
 */
function issue_2592_non_final_class_string(Issue2592Base $base, string $class): Issue2592Base
{
    if ($base instanceof $class) {
        return $base;
    } else {
        return $base;
    }
}

function issue_2592_object_rhs(Issue2592Base $base, Issue2592Base $other): Issue2592Base
{
    if ($base instanceof $other) {
        return $base;
    } else {
        return $base;
    }
}

/**
 * @param class-string<Issue2592Base> $class
 */
function issue_2592_non_final_union(Issue2592Base|Issue2592Bar $value, string $class): Issue2592Base|Issue2592Bar
{
    if ($value instanceof $class) {
        return $value;
    }

    return $value;
}

/**
 * @param class-string<Issue2592Foo> $class
 */
function issue_2592_final_class_string(Issue2592Foo|Issue2592Bar $value, string $class): Issue2592Bar
{
    if ($value instanceof $class) {
        return new Issue2592Bar();
    }

    return $value;
}

function issue_2592_literal_class_string(Issue2592Foo|Issue2592Bar $value): Issue2592Bar
{
    $class = Issue2592Foo::class;
    if ($value instanceof $class) {
        return new Issue2592Bar();
    }

    return $value;
}

/**
 * @template T of object
 *
 * @param class-string<T> $class
 *
 * @return T
 *
 * @throws RuntimeException
 */
function issue_2592_template(object $value, string $class): object
{
    if ($value instanceof $class) {
        return $value;
    }

    throw new RuntimeException('Unexpected object.');
}
