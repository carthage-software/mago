<?php

declare(strict_types=1);

use BcMath\Number;

/**
 * @return list<bool>
 */
function gmp_compared_with_accepted_operands(GMP $a, GMP $b, int $i, bool $flag): array
{
    return [
        $a < $i,
        $a <= '5',
        $a > ' 5',
        $a >= '0x5',
        $a == $b,
        $a != $flag,
        $a <> 5,
        $i < $a,
        '5' == $a,
        gmp_init(5) == '5',
        '5' == gmp_init(5),
    ];
}

/**
 * @return list<-1|0|1>
 */
function gmp_spaceship_with_accepted_operands(GMP $a, GMP $b, int $i): array
{
    return [
        $a <=> $i,
        $a <=> $b,
        '-5' <=> $a,
    ];
}

/**
 * @param numeric $numeric
 * @param scalar $scalar
 *
 * @return list<bool>
 */
function number_compared_with_any_operand(
    Number $a,
    Number $b,
    int $i,
    float $f,
    string $s,
    array $array,
    object $object,
    mixed $numeric,
    mixed $scalar,
): array {
    return [
        $a < $i,
        $a <= '5.5',
        $a == $b,
        $a != $s,
        $a == $array,
        $a == new stdClass(),
        $a < 1.5,
        $a == $f,
        1.5 > $a,
        $a < $numeric,
        $a == $scalar,
        $a < $object,
    ];
}

/**
 * @return list<-1|0|1>
 */
function number_spaceship_with_any_operand(Number $a, Number $b, int $i, float $f): array
{
    return [
        $a <=> $i,
        $a <=> $b,
        $a <=> $f,
    ];
}

/**
 * @return list<bool>
 */
function comparisons_whose_handler_accepts_the_other_operand(GMP $g, Number $n, stdClass $o): array
{
    return [
        $g > $n,
        $g >= $n,
        $n < 'abc',
        5 < $g,
        $o < $g,
    ];
}

function number_spaceship_gmp(Number $n, GMP $g): int
{
    return $n <=> $g;
}

/**
 * PHP may swap the operands of `==`, `!=` and `<>`, so with two objects which one compares is unknown.
 *
 * @return list<bool>
 */
function equality_between_two_objects(GMP $g, Number $n, stdClass $o, object $object): array
{
    return [
        $g == $n,
        $n == $g,
        $g != $o,
        $o <> $g,
        $g == $object,
        gmp_init(5) == $n,
        $n == gmp_init(5),
        new stdClass() == $g,
        $g == new stdClass(),
    ];
}

function gmp_identical_to_non_integer_string(GMP $a): bool
{
    /** @mago-expect analysis:redundant-comparison */
    return $a === 'abc';
}

function gmp_not_identical_to_array(GMP $a): bool
{
    /** @mago-expect analysis:redundant-comparison */
    return $a !== [];
}

function gmp_less_than_array(GMP $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a < [];
}

function gmp_equal_to_non_integer_string(GMP $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a == 'abc';
}

function non_integer_string_equal_to_gmp(GMP $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return 'abc' == $a;
}

function gmp_temporary_equal_to_non_integer_string(): bool
{
    /** @mago-expect analysis:invalid-operand */
    return gmp_init(5) == 'abc';
}

function non_integer_string_not_equal_to_gmp_temporary(): bool
{
    /** @mago-expect analysis:invalid-operand */
    return 'abc' != gmp_init(5);
}

function gmp_not_equal_to_decimal_string(GMP $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a != '1.5';
}

function gmp_less_than_float(GMP $a): bool
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a < 1.5;
}

function gmp_less_than_object(GMP $a, stdClass $b): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a < $b;
}

function gmp_less_than_number(GMP $a, Number $b): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a < $b;
}

function number_greater_than_gmp(Number $a, GMP $b): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a > $b;
}

function array_less_than_gmp(GMP $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return [] < $a;
}

function non_integer_string_spaceship_gmp(GMP $a): int
{
    /** @mago-expect analysis:invalid-operand */
    return 'abc' <=> $a;
}

function gmp_spaceship_array(GMP $a): int
{
    /** @mago-expect analysis:invalid-operand */
    return $a <=> [];
}

function gmp_or_int_less_than_array(GMP|int $a): bool
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a < [];
}

function gmp_compared_with_int_or_array(GMP $a, int|array $b): bool
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a == $b;
}

function gmp_less_than_int_or_float(GMP $a, int|float $b): bool
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a < $b;
}

/**
 * @param array-key $b
 */
function gmp_less_than_array_key(GMP $a, int|string $b): bool
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a < $b;
}

/**
 * @param numeric $b
 */
function gmp_less_than_numeric(GMP $a, mixed $b): bool
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a < $b;
}

/**
 * @param scalar $b
 */
function scalar_spaceship_gmp(GMP $a, int|float|string|bool $b): int
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $b <=> $a;
}

/**
 * @param scalar $b
 */
function gmp_not_equal_to_scalar(GMP $a, int|float|string|bool $b): bool
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a != $b;
}

function object_greater_than_gmp(GMP $a, object $b): bool
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $b > $a;
}
