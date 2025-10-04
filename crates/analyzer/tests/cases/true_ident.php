<?php

class Foo
{
}

function get_foo(): null|Foo
{
    if (rand(0, 1)) {
        return new Foo();
    }

    return null;
}

function use_foo(Foo $foo): void
{
    var_dump($foo);
}

/**
 * @return scalar
 */
function get_scalar()
{
    if (rand(0, 1)) {
        return 1;
    }
    if (rand(0, 1)) {
        return 1.2;
    }
    if (rand(0, 1)) {
        return false;
    }
    if (rand(0, 1)) {
        return true;
    }

    return 'foo';
}

function use_true(true $value): void
{
    var_dump($value);
}

function use_false(false $value): void
{
    var_dump($value);
}

function use_bool(bool $value): void
{
    var_dump($value);
}

function use_string_or_int_or_float(string|int|float $value): void
{
    var_dump($value);
}

// Test 1: true === (expr) - should assert both $a and $b are not null
$a = get_foo();
$b = get_foo();
$true = true;
if (true === (null !== $a && null !== $b)) {
    use_foo($a);
    use_foo($b);
}

// Test 2: false === (expr) - should assert at least one of $a or $b is null
$a = get_foo();
$b = get_foo();
$false = false;
if (false === (null !== $a && null !== $b)) {
    // At least one is null - cannot safely use_foo here
    var_dump('one is null');
}

// Test 3: true !== (expr) - negation, should assert at least one is null
$a = get_foo();
$b = get_foo();
if (true !== (null !== $a && null !== $b)) {
    // At least one is null
    var_dump('one is null');
}

// Test 4: false !== (expr) - negation, should assert both are not null
$a = get_foo();
$b = get_foo();
if (false !== (null !== $a && null !== $b)) {
    use_foo($a);
    use_foo($b);
}

// Test 5: variable === true - should assert variable is type `true`
$scalar = get_scalar();
if ($scalar === true) {
    use_true($scalar);
}

// Test 6: variable === false - should assert variable is type `false`
$scalar = get_scalar();
if ($scalar === false) {
    use_false($scalar);
}

// Test 7: variable !== true - should assert variable is not type `true`
$scalar = get_scalar();
if ($scalar !== true) {
    // $scalar is not true (could be false, string, int, float)
    var_dump($scalar);
}

// Test 8: variable !== false - should assert variable is not type `false`
$scalar = get_scalar();
if ($scalar !== false) {
    // $scalar is not false (could be true, string, int, float)
    var_dump($scalar);
}

// Test 9: true === variable - should assert variable is type `true` (reversed)
$scalar = get_scalar();
if (true === $scalar) {
    use_true($scalar);
}

// Test 10: false === variable - should assert variable is type `false` (reversed)
$scalar = get_scalar();
if (false === $scalar) {
    use_false($scalar);
}

// Test 11: Complex - true === (expr with scalar)
$scalar1 = get_scalar();
$scalar2 = get_scalar();
if (true === ($scalar1 === true && $scalar2 === true)) {
    use_true($scalar1);
    use_true($scalar2);
}

// Test 12: Nested complex expression
$a = get_foo();
$scalar = get_scalar();
if (true === (null !== $a && $scalar === true)) {
    use_foo($a);
    use_true($scalar);
}

/// Test 13: OR with true and false
$scalar = get_scalar();
if ($scalar === true || $scalar === false) {
    use_bool($scalar);
} else {
    use_string_or_int_or_float($scalar);
}
