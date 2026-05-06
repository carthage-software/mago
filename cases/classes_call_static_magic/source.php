<?php

declare(strict_types=1);

/**
 * @method static int build(int $n)
 */
final class ClassesCallStaticMagic
{
    /** @param array<mixed> $arguments */
    public static function __callStatic(string $name, array $arguments): int
    {
        unset($name, $arguments);
        return 0;
    }
}

echo ClassesCallStaticMagic::build(5);
