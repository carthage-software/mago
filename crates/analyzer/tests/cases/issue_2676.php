<?php

declare(strict_types=1);

namespace Issue2676;

/**
 * @template-covariant _A
 * @template-covariant _B
 */
abstract class Base
{
    /**
     * @phpstan-assert-if-true Derived<_B> $this
     */
    abstract public function isDerived(): bool;
}

/**
 * @template-covariant B
 *
 * @extends Base<never, B>
 */
final class Derived extends Base
{
    /**
     * @param B $value
     */
    public function __construct(
        public readonly mixed $value,
    ) {}

    public function isDerived(): bool
    {
        return true;
    }
}

/**
 * @param Base<int, string> $base
 */
function derived_value(Base $base): string
{
    if ($base->isDerived()) {
        return $base->value;
    }

    return '';
}
