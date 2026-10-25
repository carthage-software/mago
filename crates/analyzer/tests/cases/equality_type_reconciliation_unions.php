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
        takes_false($a); // @mago-expect analysis:invalid-argument
        takes_false($b); // @mago-expect analysis:invalid-argument
    } else {
        takes_string($a); // @mago-expect analysis:possibly-invalid-argument
        takes_string($b); // @mago-expect analysis:possibly-invalid-argument
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
        takes_false($value); // @mago-expect analysis:possibly-invalid-argument
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
        takes_false($value); // @mago-expect analysis:possibly-invalid-argument
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

function takes_true(true $_value): void {}

function test_union_equality_compound_guard(int|string $a, float|string $b, bool $flag): void {
    if ($a === $b && $flag) {
        return;
    }

    takes_false($flag); // @mago-expect analysis:possibly-invalid-argument
}

function test_union_equality_negated_disjunction(int|string $a, float|string $b, bool $flag): void {
    if (!($a === $b) || $flag) {
        takes_true($flag); // @mago-expect analysis:less-specific-argument
    }
}

function accepts_string(string $_value): bool {
    return true;
}

function test_union_equality_composition(int|string $a, float|string $b, bool $flag): void {
    if ($a === $b && accepts_string($a)) {
        takes_string($b);
    }

    if ($a === $b || $flag) {
        if (!$flag) {
            takes_string($a);
            takes_string($b);
        }
    }

    if (!!(($a) === ($b))) {
        takes_string($a);
        takes_string($b);
    }

    if (!($a === $b)) {
        return;
    }

    takes_string($a);
    takes_string($b);
}

/**
 * @param 'a'|'b' $tag
 * @param 'a'|'b'|false $value
 */
function test_union_equality_loose_boolean_wrapper(string $tag, string|false $value): void {
    if ((!($tag === $value)) == true) {
        takes_false($value); // @mago-expect analysis:possibly-invalid-argument
    }
}

function test_union_equality_result_assignment(int|string $a, float|string $b): void {
    if ($a = ($a === $b)) {
        takes_true($a);
        takes_string($b);
        takes_string($a); // @mago-expect analysis:invalid-argument
    }
}

function test_union_equality_operand_assignment(int|string $x, float|string $b, int $replacement): void {
    if (($a = $x) === ((($a = $replacement) > 0) ? $b : $b)) {
        takes_string($a); // @mago-expect analysis:invalid-argument
    }
}

/**
 * @param array<string, int>|bool $a
 * @param array<string, string>|bool $b
 */
function test_union_equality_empty_arrays(array|bool $a, array|bool $b): bool {
    if ($a === $b) {
        return $a; // @mago-expect analysis:invalid-return-statement
    }

    return false;
}

test_union_equality_empty_arrays([], []);

/**
 * @param non-empty-array<string, int>|bool $a
 * @param array<string, string>|bool $b
 */
function test_union_equality_non_empty_array(array|bool $a, array|bool $b): bool {
    if ($a === $b) {
        return $a;
    }

    return false;
}
