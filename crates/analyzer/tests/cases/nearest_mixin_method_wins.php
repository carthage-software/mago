<?php

declare(strict_types=1);

final class Deep
{
    public function foo(): int
    {
        return 1;
    }
}

/**
 * @mixin Deep
 */
final class Near
{
    public function foo(): string
    {
        return '';
    }

    /** @param list<mixed> $arguments */
    public function __call(string $name, array $arguments): mixed
    {
        return (new Deep())->foo();
    }
}

/**
 * @mixin Near
 */
final class Host
{
    /** @param list<mixed> $arguments */
    public function __call(string $name, array $arguments): mixed
    {
        return (new Near())->foo();
    }
}

function f(): string
{
    return (new Host())->foo();
}
