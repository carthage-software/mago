<?php

declare(strict_types=1);

function issue_2229_string_to_number(string $type): float|int
{
    return is_numeric($type) ? +$type : 0;
}

function issue_2229_number_string(string $type): int|string
{
    return is_numeric($type) ? '' . +$type : 0;
}

const ISSUE_2229_NUMBER = +'1';

function issue_2229_constant_number(): float|int
{
    return ISSUE_2229_NUMBER;
}

/** @param numeric-string $value */
function issue_2229_negate(string $value): float|int
{
    return -$value;
}

/** @param numeric-string $value */
function issue_2229_pre_increment(string $value): float|int
{
    return ++$value;
}

/**
 * @param numeric-string $value
 * @return numeric-string
 */
function issue_2229_post_increment_result(string $value): string
{
    return $value++;
}

/** @param numeric-string $value */
function issue_2229_post_increment_value(string $value): float|int
{
    $value++;

    return $value;
}

/** @param numeric-string $value */
function issue_2229_pre_decrement(string $value): float|int
{
    return --$value;
}

/**
 * @param numeric-string $value
 * @return numeric-string
 */
function issue_2229_post_decrement_result(string $value): string
{
    return $value--;
}

/** @param numeric-string $value */
function issue_2229_post_decrement_value(string $value): float|int
{
    $value--;

    return $value;
}

const ISSUE_2229_NEGATED_NUMBER = -'1';

function issue_2229_negated_constant(): float|int
{
    return ISSUE_2229_NEGATED_NUMBER;
}
