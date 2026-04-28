<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenBoxF
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }
}

/**
 * @template T
 *
 * @param GenBoxF<T> $b
 *
 * @return T
 */
function gen_unwrap(GenBoxF $b): mixed
{
    return $b->value;
}

function takes_int_unwrap(int $n): void
{
}

/**
 * @var GenBoxF<int> $b
 *
 * @mago-expect analysis:redundant-docblock-type
 */
$b = new GenBoxF(7);
takes_int_unwrap(gen_unwrap($b));
