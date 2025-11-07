<?php

/** @param literal-int $v */
function foo(int $v): void
{
    echo $v;
}

// Test 1: Passing literal integers directly - should be OK
foo(1); // ok
foo(42); // ok
foo(-5); // ok
foo(0); // ok

// Test 2: Passing a function that preserves literal-int
/** @param literal-int $v */
function bar(int $v): void
{
    foo($v); // ok - $v is literal-int
}

// Test 3: Passing a function that doesn't preserve literal-int
function baz(int $v): void
{
    foo($v); // @mago-expect analysis:possibly-invalid-argument
}

// Test 4: Variable with literal value
$x = 5;
foo($x); // ok - $x has literal value 5

// Test 5: Variable with dynamic value
$y = rand(1, 10);
foo($y); // @mago-expect analysis:invalid-argument

// Test 6: Union of literals
$z = rand() ? 5 : 10;
foo($z); // ok - union of literals is still literal

// Test 7: Mixed with non-literal
function getValue(): int
{
    return 42;
}

$w = rand() ? 5 : getValue();
foo($w); // @mago-expect analysis:possibly-invalid-argument

// Test 8: Arithmetic on literals produces literal
$a = 5;
$b = 10;
$c = $a + $b; // $c is int(15) which is literal
foo($c); // ok

// Test 9: Function that returns literal-int
/** @return literal-int */
function getLiteralInt(): int
{
    return 42; // ok - returning literal
}

foo(getLiteralInt()); // ok

// Test 10: Function that incorrectly claims to return literal-int
/** @return literal-int */
function badGetLiteralInt(int $param): int
{
    return $param; // @mago-expect analysis:invalid-return-statement
}

// Test 11: Array access
$arr = [1, 2, 3];
foo($arr[0]); // ok if analyzer knows array values are literals

// Test 12: Class constant
class MyClass
{
    const MY_CONST = 123;
}

foo(MyClass::MY_CONST); // ok - const is literal

function get_bool(): bool
{
    return rand(0, 1) === 1;
}

// Test 13: Conditional with literals
$cond = get_bool() ? 5 : 10;
foo($cond); // ok - both branches are literals

// Test 14: Loop variable
for ($i = 0; $i < 10; $i++) {
    foo($i); // @mago-expect analysis:invalid-argument
}

// Test 15: Literal-int parameter in closure
$closure =
    /**
     * @param literal-int $x
     */
    function (int $x) use ($y): void {
        foo($x); // ok - $x is literal-int
        foo($y); // @mago-expect analysis:invalid-argument
    };

$closure(100); // ok
