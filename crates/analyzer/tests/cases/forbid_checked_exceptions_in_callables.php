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
function defer_callback(Closure $callback): void
{
}

/** @param callable(): void $callback */
function defer_any(callable $callback): void
{
}

/** @param array<callable(): void> $callbacks */
function defer_array(array $callbacks): void
{
}

/** @param Closure(): void $callback */
function invoke_callback(Closure $callback): void
{
    $callback();
}

/** @param Closure(): void $callback */
function store_and_invoke(Closure $callback): void
{
    static $stored;
    $stored = $callback;
    $callback();
}

/** @param Closure(): void $callback */
function trusted_callbacks(Closure $callback): void
{
}

final class TrustedDispatcher
{
    /** @param Closure(): void $callback */
    public static function consume(Closure $callback): void
    {
    }
}

final class DeferredCallableProvider
{
    /** @throws DeferredFailure */
    public static function run(): void
    {
        fail_deferred();
    }
}

/** @throws DeferredFailure */
function named_deferred_callback(): void
{
    fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
}

function exercise_deferred_callable_checks(): void
{
    $stored = /** @throws DeferredFailure */
        static function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        };

    defer_callback(/** @throws DeferredFailure */
        static function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        });

    defer_callback(static function (): void {
        try {
            fail_deferred();
        } catch (DeferredFailure) {
        }
    });

    TrustedDispatcher::consume(/** @throws DeferredFailure */
        static function (): void {
            fail_deferred();
        });

    trusted_callbacks(/** @throws DeferredFailure */
        static function (): void {
            fail_deferred();
        });

    $callbacks = [
        static function (): void {
            fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
        },
    ];

    defer_array($callbacks);
    defer_any(named_deferred_callback(...));

    $stored_method = [DeferredCallableProvider::class, 'run'];

    try {
        DeferredCallableProvider::run();
    } catch (DeferredFailure) {
    }

    try {
        array_map(
            /** @throws DeferredFailure */
            static function (int $value): int {
                fail_deferred();

                return $value;
            },
            [1],
        );
    } catch (DeferredFailure) {
    }

    try {
        invoke_callback(/** @throws DeferredFailure */
            static function (): void {
                fail_deferred();
            });
    } catch (DeferredFailure) {
    }

    try {
        store_and_invoke(/** @throws DeferredFailure */
            static function (): void {
                fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
            });
    } catch (DeferredFailure) {
    }
}

exercise_deferred_callable_checks();
