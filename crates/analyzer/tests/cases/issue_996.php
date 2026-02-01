<?php

declare(strict_types=1);

interface Foo {}

interface Baz
{
    public int $baz { get; }
}

interface Qux
{
    public string $qux { get; }
}

/**
 * @param Foo $obj
 */
function test_single_intersection(Foo $obj): int
{
    if ($obj instanceof Baz) {
        return $obj->baz;
    }

    return 0;
}

/**
 * @param Foo $obj
 */
function test_multiple_intersections(Foo $obj): string
{
    if ($obj instanceof Baz && $obj instanceof Qux) {
        $result = $obj->baz;

        return $obj->qux . (string) $result;
    }

    return '';
}

/**
 * @param Foo $obj
 */
function test_nested_instanceof(Foo $obj): int
{
    if ($obj instanceof Baz) {
        if ($obj instanceof Qux) {
            $_ = $obj->qux;
        }

        return $obj->baz;
    }

    return 0;
}

class MyClass implements Foo, Baz
{
    public int $baz {
        get => 42;
    }
}

function test_with_concrete_class(MyClass $obj): int
{
    return $obj->baz;
}

abstract class AbstractWithProperty implements Foo
{
    public int $value;
}

function test_with_abstract_class(Foo $obj): int
{
    if ($obj instanceof AbstractWithProperty) {
        return $obj->value;
    }

    return 0;
}
