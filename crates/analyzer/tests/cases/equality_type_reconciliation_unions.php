<?php

declare(strict_types=1);

function takes_string(string $_value): void {}

function takes_false(false $_value): void {}

/**
 * @param int|string $a
 * @param float|string $b
 */
function test_union_equality_narrowing(int|string $a, float|string $b): void {
    if ($a === $b) {
        takes_string($a);
        takes_string($b);
    }
}

/**
 * @param 'a'|'b' $tag
 * @param 'a'|'b'|false $value
 */
function test_union_equality_false_branch(string $tag, string|false $value): void {
    if ($tag === $value) {
        takes_string($tag);
        takes_string($value);
    } else {
        if ($value === 'a') {
            takes_string($value);
        }
        if ($value === 'b') {
            takes_string($value);
        }
        if ($value === false) {
            takes_false($value);
        }
    }
}

/**
 * @param 'a'|'b' $tag
 * @param 'a'|'b'|false $value
 */
function test_union_equality_negated_condition(string $tag, string|false $value): void {
    if (!($tag === $value)) {
        if ($value === 'a') {
            takes_string($value);
        }
        if ($value === 'b') {
            takes_string($value);
        }
        if ($value === false) {
            takes_false($value);
        }
    }
}

/**
 * @return int|string
 */
function get_int_or_string(): int|string {
    return 1;
}

function test_untrackable_expressions(): void {
    if (get_int_or_string() === get_int_or_string()) {
    }
}

/**
 * @param int|string $a
 * @param int|string $b
 */
function test_equal_unions(int|string $a, int|string $b): void {
    if ($a === $b) {
    }
}
