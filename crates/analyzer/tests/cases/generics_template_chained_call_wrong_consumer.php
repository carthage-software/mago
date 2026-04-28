<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenChainCall2
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

    /**
     * @template U
     *
     * @param U $value
     *
     * @return GenChainCall2<U>
     */
    public static function of(mixed $value): GenChainCall2
    {
        return new GenChainCall2($value);
    }
}

function takes_int_chain2(int $n): void
{
}

/** @mago-expect analysis:invalid-argument */
takes_int_chain2(GenChainCall2::of('hi')->get());
