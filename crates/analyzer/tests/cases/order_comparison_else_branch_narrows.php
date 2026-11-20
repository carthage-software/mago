<?php

declare(strict_types=1);

/**
 * @return positive-int
 */
function order_comparison_with_literal_variable(int $y): int
{
    $z = 7;
    if ($y < $z) {
        return 1;
    }

    return $y;
}

/**
 * @param list<int> $a
 *
 * @return list<never>
 */
function order_comparison_on_count(array $a): array
{
    if (count($a) > 0) {
        return [];
    }

    return $a;
}

/**
 * @param array<string, int>|Countable $a
 *
 * @return array{}|Countable
 */
function order_comparison_on_count_keeps_countable(array|Countable $a): array|Countable
{
    if (0 < count($a)) {
        return [];
    }

    return $a;
}

/**
 * @return ''
 */
function order_comparison_on_string_length(string $s): string
{
    if (strlen($s) < 1) {
        return $s;
    }

    return '';
}

/**
 * @return ''
 */
function negated_order_comparison_on_string_length(string $s): string
{
    if (strlen($s) > 0) {
        return '';
    }

    return $s;
}
