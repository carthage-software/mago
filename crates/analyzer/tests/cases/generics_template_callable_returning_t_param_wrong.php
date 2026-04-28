<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param callable(): T $fn
 *
 * @return T
 */
function gen_call_factory_x(callable $fn): mixed
{
    return $fn();
}

function take_str_call_x(string $s): void
{
}

/** @mago-expect analysis:invalid-argument */
take_str_call_x(gen_call_factory_x(static fn(): int => 5));
