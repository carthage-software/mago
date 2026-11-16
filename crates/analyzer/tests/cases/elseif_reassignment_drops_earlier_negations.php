<?php

declare(strict_types=1);

function load(): ?int
{
    return rand(0, 1) === 1 ? null : 1;
}

function after_chain(?int $value): ?int
{
    if ($value === null) {
        return 0;
    } elseif (($value = load()) !== null) {
        return 1;
    }

    return $value;
}

function else_branch(?int $value): ?int
{
    if ($value === null) {
        return 0;
    } elseif ($value = load()) {
        return 1;
    } else {
        return $value;
    }
}

function wrong_return(?int $value): int
{
    if ($value === null) {
        return 0;
    } elseif (($value = load()) !== null) {
        return 1;
    }

    return $value; // @mago-expect analysis:invalid-return-statement
}
