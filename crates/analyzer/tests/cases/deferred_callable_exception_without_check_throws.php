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

function store_documented_callable(): void
{
    $stored = /**
             * @throws DeferredFailure
             */
        function (): void {
            fail_deferred();
        };
}

store_documented_callable();
