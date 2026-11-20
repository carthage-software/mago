<?php

declare(strict_types=1);

/**
 * @param Closure(int): void $callback
 *
 * @return non-empty-string
 */
function closure_class_constant(Closure $callback): string
{
    return $callback::class;
}

/**
 * @return non-empty-string
 */
function arrow_function_class_constant(): string
{
    $callback = static fn(): int => 1;

    return $callback::class;
}
