<?php

declare(strict_types=1);

/** @throws RuntimeException */
function may_throw(): int
{
    if (rand(0, 1) === 1) {
        throw new RuntimeException();
    }

    return rand(0, 1);
}

function captured_by_use(): ?int
{
    try {
        $y = may_throw();
    } catch (RuntimeException) {
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    $f = function () use ($y): ?int {
        $copy = $y;

        return $y ?? $copy;
    };

    return $f();
}

function captured_by_arrow(bool $flag): int
{
    if ($flag) {
        $y = 1;
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    $f = fn(): int => (int) $y;

    return $f();
}
