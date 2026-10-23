<?php

final class DeferredFailure extends Exception
{
}

/** @throws DeferredFailure */
function fail_deferred(): void
{
    throw new DeferredFailure();
}

/** @param Closure(): void $callback */
function store_nested_callback_and_invoke(Closure $callback): void
{
    $GLOBALS['callbacks'] = [$callback];
    $callback();
}

function exercise_nested_callback_store(): void
{
    store_nested_callback_and_invoke(/** @throws DeferredFailure */
        static function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        });
}

function exercise_later_immediate_helper(): void
{
    invoke_callback_declared_later(/** @throws DeferredFailure */
        static function (): void {
            fail_deferred();
        });

    invoke_callback_declared_later(static function (): void {
        fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
    });
}

/** @param Closure(): void $callback */
function invoke_callback_declared_later(Closure $callback): void
{
    $callback();
}

exercise_nested_callback_store();
exercise_later_immediate_helper();
