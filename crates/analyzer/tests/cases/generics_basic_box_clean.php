<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenBoxBasic
{
    /**
     * @param T $value
     */
    public function __construct(public mixed $value)
    {
    }

    /**
     * @return T
     */
    public function get(): mixed
    {
        return $this->value;
    }
}

/**
 * @param GenBoxBasic<int> $box
 */
function take_int_box(GenBoxBasic $box): int
{
    return $box->get();
}

take_int_box(new GenBoxBasic(42));
