<?php

declare(strict_types=1);

final class RecursiveSpecificFailure extends RuntimeException
{
}

/**
 * @template TFailure of Throwable
 * @param TFailure $failure
 * @throws TFailure
 */
function recursive_generic_throw_resolution(Throwable $failure): void
{
    if ($failure instanceof RecursiveSpecificFailure) {
        recursive_generic_throw_resolution(new RecursiveSpecificFailure());
    }
}

function createBroadThrowableForRecursiveThrow(): Throwable
{
    return new RuntimeException();
}

/**
 * @template TFailure of Throwable
 * @param TFailure $failure
 * @throws TFailure
 */
function recursive_generic_throw_resolution_with_broad_binding(Throwable $failure): void
{
    recursive_generic_throw_resolution_with_broad_binding(createBroadThrowableForRecursiveThrow());
}
