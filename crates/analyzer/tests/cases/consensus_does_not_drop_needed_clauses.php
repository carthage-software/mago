<?php

declare(strict_types=1);

function consensus_three_reduced(mixed $a, mixed $x, mixed $y): string
{
    if (
        (is_int($a) || is_int($x) || is_int($y))
        && (!is_int($a) || is_int($x) || is_int($y))
        && (is_int($a) || !is_int($x) || is_int($y))
    ) {
        if (is_int($y)) {
            return 'int';
        }
    }

    return 'other';
}

function consensus_two_clashing_variables(mixed $a, mixed $b, mixed $x, mixed $y): string
{
    if (
        (is_int($a) || is_int($b) || is_int($x))
        && (!is_int($a) || !is_int($b) || is_int($y))
        && (is_int($x) || is_int($y))
    ) {
        return 'if';
    } elseif (is_int($a)) {
        if (is_int($b)) {
            return 'int';
        }
    }

    return 'other';
}

function consensus_removed_parent(?int $p, ?int $u, ?int $v): string
{
    // @mago-expect analysis:redundant-logical-operation(2),redundant-comparison
    if (
        ($p !== null || $v !== null)
        && ($v === null || $u !== null)
        && ($p !== null || $u !== null)
        && ($u === null || $v !== null)
    ) {
        return 'if';
    }

    if ($u === null) {
        if ($v === null) {
            return 'null';
        }
    }

    return 'other';
}

function consensus_removed_parent_reordered(?int $p, ?int $u, ?int $v): string
{
    // @mago-expect analysis:redundant-logical-operation(2),redundant-comparison
    if (
        ($v !== null || $p !== null)
        && ($v === null || $u !== null)
        && ($p !== null || $u !== null)
        && ($u === null || $v !== null)
    ) {
        return 'if';
    } elseif ($u === null) {
        if ($v === null) {
            return 'null';
        }
    }

    return 'other';
}

function consensus_removed_later_parent(?int $p, ?int $u, ?int $v): string
{
    // @mago-expect analysis:redundant-logical-operation(2),redundant-comparison
    if (
        ($u === null || $v !== null)
        && ($v === null || $u !== null)
        && ($p !== null || $u !== null)
        && ($v !== null || $p !== null)
    ) {
        return 'if';
    } elseif ($u === null) {
        if ($v === null) {
            return 'null';
        }
    }

    return 'other';
}

function consensus_unreconcilable_parent(?int $a, ?int $x, ?int $y, string $s, string $t): string
{
    if (($a !== null || $x !== null || !($s === $t)) && ($a === null || $y !== null) && ($x !== null || $y !== null)) {
        return 'if';
    }

    if ($a === null) {
        return 'null';
    }

    return 'other';
}

function consensus_unreconcilable_later_parent(?int $a, ?int $x, ?int $y, string $s, string $t): string
{
    if (($a === null || $y !== null) && ($a !== null || $x !== null || !($s === $t)) && ($x !== null || $y !== null)) {
        return 'if';
    }

    if ($a === null) {
        return 'null';
    }

    return 'other';
}
