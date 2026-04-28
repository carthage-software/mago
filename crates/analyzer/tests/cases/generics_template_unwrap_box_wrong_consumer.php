<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenBoxFW
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }
}

/**
 * @template T
 *
 * @param GenBoxFW<T> $b
 *
 * @return T
 */
function gen_unwrap2(GenBoxFW $b): mixed
{
    return $b->value;
}

function takes_str_unwrap(string $s): void
{
}

/**
 * @param GenBoxFW<int> $b
 */
function bridge_unwrap(GenBoxFW $b): void
{
    /** @mago-expect analysis:invalid-argument */
    takes_str_unwrap(gen_unwrap2($b));
}
