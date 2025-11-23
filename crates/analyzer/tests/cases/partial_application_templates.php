<?php

/**
 * @param list{int, int} $_
 */
function accept_int_tuple(array $_): void {}

/**
 * @param list{string, string} $_
 */
function accept_string_tuple(array $_): void {}

/**
 * @template T
 *
 * @param T $value
 *
 * @return list{T, T}
 */
function to_tuple($value): array
{
    return [$value, $value];
}

/**
 * @template T
 *
 * @param T $a
 * @param T $b
 *
 * @return list{T, T}
 */
function pair($a, $b): array
{
    return [$a, $b];
}

$to_tuple_fcc = to_tuple(...);
$result = $to_tuple_fcc(42);
accept_int_tuple($result);

$result = $to_tuple_fcc("hello");
accept_string_tuple($result);


$pair_with_5 = pair(5, ?);
$result = $pair_with_5(5);
accept_int_tuple($result);

$result = $pair_with_5(10); // @mago-expect analysis:invalid-argument - expects int 5, got int 10
accept_int_tuple($result);

$pair_partial = pair(?, "world");
$result = $pair_partial("world");
accept_string_tuple($result);

$result = $pair_partial("hello"); // @mago-expect analysis:invalid-argument - expects string "world", got string "hello"
accept_string_tuple($result);

to_tuple(?)(1, 2); // @mago-expect analysis:too-many-arguments
pair(?, ?)(); // @mago-expect analysis:too-few-arguments
pair(?, ?)(1); // @mago-expect analysis:too-few-arguments analysis:invalid-argument,invalid-argument
