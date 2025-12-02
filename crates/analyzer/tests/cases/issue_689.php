<?php

/**
 * @param string $value
 */
function accepts_string(string $value): void
{
    echo $value;
}

class Container
{
    public mixed $out;
}

// Test case 1: Simple is_string on mixed variable (should work)
function test_simple_is_string(mixed $x): void
{
    if (is_string($x)) {
        accepts_string($x);
    }
}

// Test case 2: isset + is_string on mixed variable (should work)
function test_isset_and_is_string(mixed $x): void
{
    if (isset($x) && is_string($x)) {
        accepts_string($x);
    }
}

// Test case 3: is_string on object property with typed property (should work)
function test_is_string_on_property(Container $c): void
{
    if (is_string($c->out)) {
        accepts_string($c->out);
    }
}

// Test case 4: isset + is_string on object property (THE ORIGINAL BUG)
// The bug was: when combining isset($c->out) && is_string($c->out),
// the type of $c->out was narrowed to nonnull instead of string
function test_isset_and_is_string_on_property(Container $c): void
{
    if (isset($c->out) && is_string($c->out)) {
        // Here $c->out should be string, not nonnull
        accepts_string($c->out);
    }
}

function main(): void
{
    $a = '{"out": "123"}';
    /** @var mixed **/
    $c = \json_decode($a, flags: \JSON_THROW_ON_ERROR);
    if (isset($c->out) && \is_string($c->out)) {
        accepts_string($c->out);
    }
}
