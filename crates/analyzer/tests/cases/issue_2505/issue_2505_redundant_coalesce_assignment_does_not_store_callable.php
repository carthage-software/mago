<?php

final class DeferredCoalesceFailure extends Exception
{
}

function redundant_coalesce_assignment_does_not_store_callable(): void
{
    $callback = static fn (): null => null;
    $callback ??= /** @throws DeferredCoalesceFailure */ static function (): void {
        throw new DeferredCoalesceFailure();
    };

    static $staticCallback;
    $staticCallback = static fn (): null => null;
    $staticCallback ??= /** @throws DeferredCoalesceFailure */ static function (): void {
        throw new DeferredCoalesceFailure();
    };
}

redundant_coalesce_assignment_does_not_store_callable();
