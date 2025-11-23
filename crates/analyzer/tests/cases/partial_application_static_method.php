<?php

/** @param int $_ */
function accept_int(int $_): void {}

/** @param string $_ */
function accept_string(string $_): void {}

class Math {
    public static function add(int $a, int $b): int {
        return $a + $b;
    }

    public static function multiply(int $a, int $b, int $c): int {
        return $a * $b * $c;
    }
}

$add_static_fcc = Math::add(...);
$result1 = $add_static_fcc(10, 5);
accept_int($result1);

$add_ten = Math::add(10, ?);
$result2 = $add_ten(5);
accept_int($result2);

$multiply_partial = Math::multiply(?, 5, ?);
$result4 = $multiply_partial(2, 3);
accept_int($result4);

class Util {
    public static function concat(string ...$parts): string {
        return implode("", $parts);
    }
}

$concat_fcc = Util::concat(...);
$result5 = $concat_fcc("a", "b", "c");
accept_string($result5);

Math::add(?)(1); // @mago-expect analysis:too-few-arguments
Math::add(?, ?)(1); // @mago-expect analysis:too-few-arguments
Math::multiply(?, ?, ?)(); // @mago-expect analysis:too-few-arguments
Util::concat(...)();
Math::add(?)(1, 2, 3); // @mago-expect analysis:too-few-arguments,too-many-arguments
