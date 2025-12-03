<?php

class Foo
{
}

class Bar
{
}

class Baz
{
}

/**
 * @psalm-assert Bar|Baz $value
 */
function assert_is_bar_or_baz(mixed $value): void
{
}

/**
 * @psalm-assert Bar $value
 */
function assert_is_bar(mixed $value): void
{
}

/**
 * @mago-expect analysis:impossible-type-comparison
 */
function test_impossible_assertion(): void
{
    $foo = new Foo();
    assert_is_bar_or_baz($foo);
}

/**
 * @mago-expect analysis:impossible-type-comparison
 */
function test_impossible_assertion_single(): void
{
    $foo = new Foo();
    assert_is_bar($foo);
}

function test_valid_assertion_bar(): void
{
    $bar = new Bar();
    assert_is_bar_or_baz($bar);
}

function test_valid_assertion_baz(): void
{
    $baz = new Baz();
    assert_is_bar_or_baz($baz);
}

function test_valid_assertion_mixed(): void
{
    /** @var mixed $value */
    $value = get_value();
    assert_is_bar_or_baz($value);
}

function get_value(): mixed
{
    return new Bar();
}
