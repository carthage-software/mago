<?php

declare(strict_types=1);

/**
 * @template T
 */
abstract class GenContainerBase
{
    /** @var T */
    public mixed $value;

    /** @param T $value */
    public function __construct(mixed $value)
    {
        $this->value = $value;
    }

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

/**
 * @extends GenContainerBase<int>
 */
final class GenIntContainer extends GenContainerBase
{
}

function take_int(int $n): void
{
}

$c = new GenIntContainer(42);
take_int($c->get());
