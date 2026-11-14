<?php

declare(strict_types=1);

/** @template TValue */
final class Box
{
    /** @param TValue $value */
    public function __construct(
        /** @var TValue */
        public mixed $value,
    ) {}
}

/**
 * @template T
 * @mixin Box<T>
 */
final class Host
{
    public function __get(string $name): mixed
    {
        return null;
    }
}

/** @param Host<int> $host */
function value(Host $host): int
{
    return $host->value;
}

/**
 * @mixin Box<string>
 */
final class StringHost
{
    public function __get(string $name): mixed
    {
        return null;
    }
}

function text(StringHost $host): string
{
    return $host->value;
}
