<?php

declare(strict_types=1);

function foo(string $s, int $i): void
{
    echo "$s $i\n";
}

function bar(string $a, int $b, float $c): void
{
    echo "$a $b $c\n";
}

function baz(string $x, int $y, bool $z): void
{
    if ($z) {
        echo "$x $y true\n";
    } else {
        echo "$x $y false\n";
    }
}

function qux(int $a, string $b, float $c, bool $d): void
{
    if ($d) {
        echo "$a $b $c true\n";
    } else {
        echo "$a $b $c false\n";
    }
}

function reverse(int $first, string $second, bool $third): void
{
    if ($third) {
        echo "$first $second true\n";
    } else {
        echo "$first $second false\n";
    }
}


$f1 = foo(i: ?, s: ?);  // closure(int, string)
$f1("foo", 123); // @mago-expect analysis:invalid-argument,invalid-argument
$f1(123, "foo"); // OK - correct types for the flipped order

$f2 = bar(c: ?, a: ?, b: ?);  // closure(float, string, int)
$f2(1.5, "test", 42); // OK
$f2("test", 1.5, 42); // @mago-expect analysis:invalid-argument,invalid-argument

$f3 = baz("fixed", z: ?, y: ?);  // closure(bool, int)
$f3(true, 42); // OK
$f3(42, true); // @mago-expect analysis:invalid-argument,invalid-argument

$f4 = qux(c: ?, a: 10, d: ?, b: "test");  // closure(float, bool)
$f4(3.14, true); // OK
$f4(true, 3.14); // @mago-expect analysis:invalid-argument,invalid-argument

$f5 = reverse(third: ?, second: ?, first: ?);  // closure(bool, string, int)
$f5(true, "hello", 42); // OK
$f5(42, "hello", true); // @mago-expect analysis:invalid-argument,invalid-argument
