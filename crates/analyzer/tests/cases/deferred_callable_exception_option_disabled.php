<?php

final class DeferredFailure extends Exception
{
}

/**
 * @throws DeferredFailure
 */
function fail_deferred(): void
{
    throw new DeferredFailure();
}

/**
 * @param Closure(): void $callback
 */
function unknown_callback(Closure $callback): void
{
}

function store_documented_callable(): void
{
    $stored = /**
             * @throws DeferredFailure
             */
        function (): void {
            fail_deferred();
        };
}

function check_throws_still_reports_undocumented_callback(): void
{
    unknown_callback(function (): void {
        fail_deferred(); // @mago-expect analysis:unhandled-thrown-type
    });
}

store_documented_callable();
check_throws_still_reports_undocumented_callback();
