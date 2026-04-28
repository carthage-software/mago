<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param callable(): T $fn
 *
 * @return T
 */
function gen_call_factory2(callable $fn): mixed
{
    return $fn();
}

function takes_int_call2(int $n): void
{
}

/** @mago-expect analysis:invalid-argument */
takes_int_call2(gen_call_factory2(static fn(): string => 'hi'));
