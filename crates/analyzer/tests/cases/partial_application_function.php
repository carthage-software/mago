<?php

/** @param int $_ */
function accept_int(int $_): void {}

/** @param string $_ */
function accept_string(string $_): void {}

/** @param float $_ */
function accept_float(float $_): void {}

function add(int $a, int $b): int {
    return $a + $b;
}

function concat(string $a, string $b, string $c): string {
    return $a . $b . $c;
}

$add_fcc = add(...);
$add_result = $add_fcc(1, 2);
accept_int($add_result);

$add_one = add(1, ?);
$result1 = $add_one(2);
accept_int($result1);

$concat_partial = concat(?, "middle", ?);
$result2 = $concat_partial("first", "last");
accept_string($result2);

function sum(int ...$numbers): int {
    return array_sum($numbers);
}

$sum_fcc = sum(...);
$sum_result = $sum_fcc(1, 2, 3);
accept_int($sum_result);

function divide(int $numerator, int $denominator): float {
    return $numerator / $denominator;
}

$reciprocal = divide(denominator: ?, numerator: 1);
$result4 = $reciprocal(2);
accept_float($result4);

strlen(?)(1); // @mago-expect analysis:invalid-argument
strlen(?)("f", 2); // @mago-expect analysis:too-many-arguments
strlen(?)(); // @mago-expect analysis:too-few-arguments
add(?)(1); // @mago-expect analysis:too-few-arguments
add(?, ?)(1); // @mago-expect analysis:too-few-arguments
concat(?, ?, ?)("a"); // @mago-expect analysis:too-few-arguments
// @mago-expect analysis:too-many-arguments
// @mago-expect analysis:unused-statement
strtoupper(?, ?, ?, ?);
