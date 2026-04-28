<?php

declare(strict_types=1);

final class Holder
{
    /** @var Closure(int): int */
    public Closure $fn;

    public function __construct()
    {
        $this->fn = fn(int $n): int => $n * 2;
    }

    public function run(int $n): int
    {
        return ($this->fn)($n);
    }
}

echo (new Holder())->run(5);
