<?php

declare(strict_types=1);

function accept_int(int $_): void {}

/** @param list<int|null> $haystack */
function generic_list(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        // @mago-expect analysis:possibly-null-argument
        accept_int($needle);
    }
}

/** @param non-empty-list<int|null> $haystack */
function non_empty_list(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        // @mago-expect analysis:possibly-null-argument
        accept_int($needle);
    }
}

/** @param list{int|null} $haystack */
function tuple_with_union_element(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        // @mago-expect analysis:possibly-null-argument
        accept_int($needle);
    }
}

/** @param array{value: int|null} $haystack */
function shape_with_union_element(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        // @mago-expect analysis:possibly-null-argument
        accept_int($needle);
    }
}

/** @param array{value: 1|null} $haystack */
function shape_with_literal_union(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        // @mago-expect analysis:possibly-null-argument
        accept_int($needle);
    }
}

/** @param array{value?: null} $haystack */
function optional_null(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        // @mago-expect analysis:possibly-null-argument
        accept_int($needle);
    }
}

/** @param array{left: null}|array{right: int} $haystack */
function union_of_shapes(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        // @mago-expect analysis:possibly-null-argument
        accept_int($needle);
    }
}

/** @param list<int> $haystack */
function positive_membership(?int $needle, array $haystack): void
{
    if (in_array($needle, $haystack, true)) {
        accept_int($needle);
    }
}

/** @param list<int> $haystack */
function positive_membership_from_mixed(mixed $needle, array $haystack): void
{
    if (in_array($needle, $haystack, true)) {
        accept_int($needle);
    }
}

function required_null(?int $needle): void
{
    if (!in_array($needle, [null, 1], true)) {
        accept_int($needle);
    }
}

/** @param array{value: null, ...<array-key, int>} $haystack */
function required_null_in_open_shape(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        accept_int($needle);
    }
}

/** @param array{left: null}|array{right: null} $haystack */
function required_null_in_union_of_shapes(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        accept_int($needle);
    }
}

/** @param 3 $_ */
function accept_three(int $_): void {}

/** @param 1|2|3 $needle */
function required_integer_literals(int $needle): void
{
    if (!in_array($needle, [1, 2], true)) {
        accept_three($needle);
    }
}

enum Choice
{
    case A;
    case B;
}

/** @param Choice::B $_ */
function accept_b(Choice $_): void {}

function required_enum_case(Choice $needle): void
{
    if (!in_array($needle, [Choice::A], true)) {
        accept_b($needle);
    }
}

/** @param true $_ */
function accept_true(bool $_): void {}

function required_false(bool $needle): void
{
    if (!in_array($needle, [false], true)) {
        accept_true($needle);
    }
}

function accept_null(null $_): void {}

function required_booleans(?bool $needle): void
{
    if (!in_array($needle, [true, false], true)) {
        accept_null($needle);
    }
}

/** @param non-empty-list<null> $haystack */
function required_null_in_non_empty_list(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        accept_int($needle);
    }
}

/** @param non-empty-array<string, null> $haystack */
function required_null_in_non_empty_array(?int $needle, array $haystack): void
{
    if (!in_array($needle, $haystack, true)) {
        accept_int($needle);
    }
}

/** @param list<string> $haystack */
function disjoint_values(int $needle, array $haystack): int
{
    if (!in_array($needle, $haystack, true)) {
        return $needle;
    }

    return 0;
}
