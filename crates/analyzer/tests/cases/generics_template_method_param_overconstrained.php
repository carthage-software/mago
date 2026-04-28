<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenMethCons
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

/** @param GenMethCons<int> $m */
function take_str_from_box(GenMethCons $m): string
{
    /** @mago-expect analysis:invalid-return-statement */
    return $m->get();
}
