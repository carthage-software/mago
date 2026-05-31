<?php

declare(strict_types=1);

namespace Repro;

final class Leaf
{
    public function leafMethod(): int
    {
        return 1;
    }
}

/** @mixin Leaf */
final class Mid
{
    /**
     * @param array<array-key, mixed> $_arguments
     */
    public function __call(string $_method, array $_arguments): mixed
    {
        return null;
    }
}

/** @mixin Mid */
final class Host
{
    /**
     * @param array<array-key, mixed> $_arguments
     */
    public function __call(string $_method, array $_arguments): mixed
    {
        return null;
    }
}

function f(Mid $mid, Host $host): void
{
    $mid->leafMethod();
    $host->leafMethod();
}
