<?php

declare(strict_types=1);

namespace Issue2434;

function fromIssue(?int $x): void
{
    // @mago-expect analysis:possibly-null-operand
    if ($x > 0 && sqrt($x)) {
        echo 'ok';
    }
}

/** @return positive-int */
function greaterThan(?int $x): int
{
    // @mago-expect analysis:possibly-null-operand
    if ($x > 0) {
        return $x;
    }

    return 1;
}

/** @return positive-int */
function greaterThanOrEqualOne(?int $x): int
{
    // @mago-expect analysis:possibly-null-operand
    if ($x >= 1) {
        return $x;
    }

    return 1;
}

/** @return positive-int */
function greaterThanOnIntOrFalse(int|false $x): int
{
    // @mago-expect analysis:possibly-false-operand
    if ($x > 0) {
        return $x;
    }

    return 1;
}

/** @return positive-int */
function flippedOperands(?int $x): int
{
    // @mago-expect analysis:possibly-null-operand
    if (0 < $x) {
        return $x;
    }

    return 1;
}

function greaterThanOrEqualZeroKeepsNull(?int $x): int
{
    // @mago-expect analysis:possibly-null-operand
    if ($x >= 0) {
        if ($x === null) {
            return 0;
        }

        return $x;
    }

    return 0;
}
