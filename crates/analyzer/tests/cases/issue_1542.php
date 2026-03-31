<?php

declare(strict_types=1);

/**
 * @return false|array<string,mixed>
 */
function get_array_or_false(): array|false
{
    return mt_rand(0, 1) ? ['type' => 'test'] : false;
}

/**
 * @return null|array<string,mixed>
 */
function get_array_or_null(): ?array
{
    return mt_rand(0, 1) ? ['type' => 'test'] : null;
}

/**
 * @return array{a: int}|false
 * @psalm-ignore-falsable-return
 */
function get_array_ignore_false(): false|array
{
    return ['a' => 1];
}

/**
 * @return array{a: int}|null
 * @psalm-ignore-nullable-return
 */
function get_array_ignore_null(): ?array
{
    return ['a' => 1];
}

// possibly-false-array-access: $o can be false
$o = get_array_or_false();
/** @mago-expect analysis:possibly-false-array-access */
/** @mago-expect analysis:possibly-undefined-array-index */
echo (string) $o['type'];

// possibly-null-array-access: $n can be null
$n = get_array_or_null();
/** @mago-expect analysis:possibly-null-array-access */
/** @mago-expect analysis:possibly-undefined-array-index */
echo (string) $n['type'];

// ignore-falsable-return: should NOT emit possibly-false-array-access
$f = get_array_ignore_false();
echo (string) $f['a'];

// ignore-nullable-return: should NOT emit possibly-null-array-access
$g = get_array_ignore_null();
echo (string) $g['a'];

// invalid-array-access: int is not array-accessible
/** @var int $i */
$i = 42;
/** @mago-expect analysis:invalid-array-access */
/** @mago-expect analysis:mixed-argument */
echo $i['foo'];

// possibly-invalid-array-access: true|array, true is not array-accessible
/** @var true|array<string, int> $t */
$t = true;
/** @mago-expect analysis:possibly-invalid-array-access */
/** @mago-expect analysis:possibly-undefined-array-index */
echo $t['foo'];

// null -> array: allowed in PHP (auto-conversion)
$w_null = null;
$w_null[1] = 1;

// false -> array: deprecated in PHP 8.1+
$w_false = false;
/** @mago-expect analysis:false-array-access */
$w_false[1] = 1;

// int -> array: fatal error
$w_int = 1;
/** @mago-expect analysis:invalid-array-access */
$w_int[1] = 1;

// float -> array: fatal error
$w_float = 1.5;
/** @mago-expect analysis:invalid-array-access */
$w_float[1] = 1;

// true -> array: fatal error
$w_true = true;
/** @mago-expect analysis:invalid-array-access */
$w_true[1] = 1;

// string[int] = x: allowed (character replacement)
$w_str = 'hello';
$w_str[0] = 'H';
