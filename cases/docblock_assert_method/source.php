<?php

declare(strict_types=1);

final class AssertHelperG
{
    /**
     * @param mixed $value
     *
     * @assert int $value
     *
     * @throws InvalidArgumentException
     */
    public static function asInt(mixed $value): void
    {
        if (!is_int($value)) {
            throw new InvalidArgumentException('not int');
        }
    }
}

function uses_assertion(mixed $v): int
{
    try {
        AssertHelperG::asInt($v);

        return $v + 1;
    } catch (InvalidArgumentException) {
        return 0;
    }
}

echo uses_assertion(5);
