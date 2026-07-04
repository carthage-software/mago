<?php

declare(strict_types=1);

/**
 * @template T as (callable(): void)|null
 * @param T $cb
 */
function issue_2037_is_callable_narrow(mixed $cb): void
{
    if (is_callable($cb)) {
        $cb();
    }
}

/**
 * @template T as (callable(): void)|null
 * @param T $cb
 */
function issue_2037_null_compare_narrow(mixed $cb): void
{
    if ($cb !== null) {
        $cb();
    }
}
