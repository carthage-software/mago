<?php

function example(string $a, int $b): void
{
    $b = (string) $b;

    echo $a . $b . "\n";
}

function example_variadic(string $a, int ...$b): void
{
    foreach ($b as $num) {
        echo $a . $num . "\n";
    }
}

example("foo", 123); // ok
example("foo", b: 123); // ok
example(a: "foo", b: 123); // ok
example(b: 123, a: "foo"); // ok
example(...["a" => "foo", "b" => 123]); // ok

example("foo", 123, c: []); // @mago-expect analysis:too-many-arguments,invalid-named-argument
example("foo", b: 123, c: []); // @mago-expect analysis:too-many-arguments,invalid-named-argument
example(a: "foo", b: 123, c: []); // @mago-expect analysis:too-many-arguments,invalid-named-argument
example(b: 123, a: "foo", c: []); // @mago-expect analysis:too-many-arguments,invalid-named-argument
example(...["a" => "foo", "b" => 123, "c" => []]); // @mago-expect analysis:too-many-arguments,invalid-named-argument

example("foo", c: []); // @mago-expect analysis:invalid-named-argument
example("foo", c: []); // @mago-expect analysis:invalid-named-argument
example(a: "foo", c: []); // @mago-expect analysis:invalid-named-argument
example(a: "foo", c: []); // @mago-expect analysis:invalid-named-argument
example(...["a" => "foo", "c" => []]); // @mago-expect analysis:invalid-named-argument

/// Providing named arguments for non-existing parameters in variadic functions is allowed

example_variadic("foo", 1, 2, 3); // ok
example_variadic("foo", b: 1, c: 2, d: 3); // ok
example_variadic(a: "foo", b: 1, c: 2, d: 3); // ok
example_variadic(b: 1, a: "foo", c: 2, d2: 3); // ok

/// Just because `...` is used, it does not mean the argument is targeting the last variadic parameter
/// it should only does that if previous parameters have been filled.

example_variadic(...["a" => "foo", "b" => 123]); // ok
example_variadic(...["a" => "foo", "b" => 123, "c" => 456]); // ok
example_variadic(...["a" => "foo", "b" => 123, "c" => 456, "d" => 789]); // ok
