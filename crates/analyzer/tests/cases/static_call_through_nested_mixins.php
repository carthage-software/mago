<?php

declare(strict_types=1);

final class Deep
{
    public static function deepOnly(): int
    {
        return 1;
    }
}

/**
 * @mixin Deep
 */
final class Near
{
    /** @param list<mixed> $arguments */
    public static function __callStatic(string $name, array $arguments): mixed
    {
        return Deep::deepOnly();
    }
}

/**
 * @mixin Near
 */
final class Host
{
    /** @param list<mixed> $arguments */
    public static function __callStatic(string $name, array $arguments): mixed
    {
        return Near::__callStatic($name, $arguments);
    }
}

function f(): int
{
    return Host::deepOnly();
}
