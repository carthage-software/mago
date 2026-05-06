<?php

declare(strict_types=1);

final class GuardH
{
    /**
     * @assert-if-true int $value
     */
    public static function isInt(mixed $value): bool
    {
        return is_int($value);
    }
}

function with_guard(mixed $v): int
{
    if (GuardH::isInt($v)) {
        return $v + 1;
    }

    return 0;
}

echo with_guard(7);
echo with_guard('x');
