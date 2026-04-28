<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param callable(): T $fn
 *
 * @return T
 */
function gen_call_factory(callable $fn): mixed
{
    return $fn();
}

function takes_int_call(int $n): void
{
}

takes_int_call(gen_call_factory(static fn(): int => 5));
