<?php

declare(strict_types=1);
/**
 * @template T of int|bool
 * @param T $x
 */
function negated_is_bool_on_template($x): int { if (!is_bool($x)) { return $x; } return 0; }
/**
 * @template T of int|true
 * @param T $x
 */
function not_true_on_template($x): int { if ($x !== true) { return $x; } return 0; }
/**
 * @template T of array-key|bool
 * @param T $x
 */
function negated_array_key_on_template($x): bool { if (!is_int($x) && !is_string($x)) { return $x; } return false; }
/** @param numeric $x */
function numeric_without_int($x): float|string { if (is_int($x)) { return 'i'; } return $x; }
/**
 * @param numeric $x
 *
 * @return int|numeric-string
 */
function numeric_without_float($x): int|string
{
    if (is_float($x)) {
        return 1;
    }

    return $x;
}
