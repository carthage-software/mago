<?php

declare(strict_types=1);

/**
 * @template T of object
 */
final class GuardBU
{
    /**
     * @param class-string<T> $class
     */
    public function __construct(private string $class) {}

    /**
     * @psalm-assert-if-true T $value
     */
    public function check(mixed $value): bool
    {
        return $value instanceof $this->class;
    }
}

class TargetBU
{
    public int $n = 0;
}

function useGuardBU(mixed $v): int
{
    $g = new GuardBU(TargetBU::class);
    if ($g->check($v)) {
        return $v->n;
    }

    return -1;
}

echo useGuardBU(new TargetBU());
