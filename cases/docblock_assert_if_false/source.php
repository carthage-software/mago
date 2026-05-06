<?php

declare(strict_types=1);

final class GuardI
{
    /**
     * @assert-if-false int $value
     */
    public static function isNotInt(mixed $value): bool
    {
        return !is_int($value);
    }
}

function with_neg_guard(mixed $v): int
{
    if (GuardI::isNotInt($v)) {
        return 0;
    }

    return $v + 1;
}

echo with_neg_guard(7);
echo with_neg_guard('x');
