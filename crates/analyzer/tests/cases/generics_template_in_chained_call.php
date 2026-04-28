<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenChainCall
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
     * @return GenChainCall<U>
     */
    public static function of(mixed $value): GenChainCall
    {
        return new GenChainCall($value);
    }
}

function takes_str_chain(string $s): void
{
}

takes_str_chain(GenChainCall::of('hi')->get());
