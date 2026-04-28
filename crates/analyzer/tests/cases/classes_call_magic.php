<?php

declare(strict_types=1);

/**
 * @method int compute(int $n)
 */
final class ClassesCallMagic
{
    /** @param array<mixed> $arguments */
    public function __call(string $name, array $arguments): int
    {
        unset($name, $arguments);
        return 0;
    }
}

echo (new ClassesCallMagic())->compute(5);
