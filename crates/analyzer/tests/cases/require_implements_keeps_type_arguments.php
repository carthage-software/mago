<?php

declare(strict_types=1);

/** @template T */
interface HasValue
{
    /** @return T */
    public function value(): mixed;
}

/**
 * @template T
 * @psalm-require-implements HasValue<T>
 */
trait UsesValue
{
    /** @return T */
    public function again(): mixed
    {
        return $this->value();
    }
}
