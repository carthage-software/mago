<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenChainable
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /**
     * @param T $value
     *
     * @return self<T>
     */
    public function with(mixed $value): self
    {
        return new self($value);
    }
}

$c = new GenChainable(1);
$c2 = $c->with(2);
echo $c2->value + 1;
