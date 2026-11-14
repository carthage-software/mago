<?php

declare(strict_types=1);

use RuntimeException;

/** @throws RuntimeException */
function then_or_throw(?string $value): string
{
    $length = $value !== null ? strlen($value) : throw new RuntimeException('missing');

    return $value . $length;
}

/** @throws RuntimeException */
function throw_or_else(?string $value): string
{
    $value === null ? throw new RuntimeException('missing') : null;

    return $value;
}

/** @throws RuntimeException */
function elvis_or_throw(?string $value): string
{
    $value ?: throw new RuntimeException('missing');

    return $value;
}

/** @throws RuntimeException */
function assigned_in_finishing_branch(bool $flag): int
{
    $flag ? ($made = 5) : throw new RuntimeException('no');

    return $made;
}

function nested_assignments(bool $first, bool $second): int
{
    $first ? ($made = 1) : ($second ? ($made = 2) : ($made = 3));

    return $made;
}
