<?php

declare(strict_types=1);

function accept_object(object $_): void {}

function accept_string(string $_): void {}

function test_is_object(callable $fn): void
{
    if (is_object($fn)) {
        accept_object($fn);
    }
}

function test_is_string(callable $fn): void
{
    if (is_string($fn)) {
        accept_string($fn);
    }
}

/** @param callable(int): string $fn */
function objectBranch(callable $fn): string
{
    if (is_object($fn)) {
        return $fn(42);
    }
    return $fn(42);
}

/** @param callable(int): string $fn */
function stringBranch(callable $fn): string
{
    if (is_string($fn)) {
        return $fn(42);
    }
    return $fn(42);
}

/** @param callable(int): string $fn */
function test_is_object_invalid_argument(callable $fn): void
{
    if (is_object($fn)) {
        // @mago-expect analysis:invalid-argument
        $fn('not-an-int');
    }
}

/** @param callable(int): string $fn */
function test_is_string_invalid_argument(callable $fn): void
{
    if (is_string($fn)) {
        // @mago-expect analysis:invalid-argument
        $fn('not-an-int');
    }
}

/** @param callable(int): string $fn */
function test_is_object_wrong_return(callable $fn): int
{
    if (is_object($fn)) {
        // @mago-expect analysis:invalid-return-statement
        return $fn(42);
    }
    // @mago-expect analysis:invalid-return-statement
    return $fn(42);
}

/** @param callable(int): string $fn */
function test_is_string_wrong_return(callable $fn): int
{
    if (is_string($fn)) {
        // @mago-expect analysis:invalid-return-statement
        return $fn(42);
    }
    // @mago-expect analysis:invalid-return-statement
    return $fn(42);
}

class InvokableObject
{
    public function __invoke(int $x): string
    {
        return (string) $x;
    }
}

function accept_closure(Closure $closure): void {}

function test_invoke_object_not_closure(callable $fn): void
{
    if (is_object($fn)) {
        // @mago-expect analysis:invalid-argument
        accept_closure($fn);
    }
}

/** @param callable(int): string $fn */
function checkObject(callable $fn): void
{
    if (is_object($fn)) {
        // @mago-expect analysis:redundant-type-comparison
        // @mago-expect analysis:redundant-condition
        if (is_object($fn)) {
            $fn(42);
        }
    }
}

/** @param callable(int): string $fn */
function checkString(callable $fn): void
{
    if (is_string($fn)) {
        // @mago-expect analysis:redundant-type-comparison
        // @mago-expect analysis:redundant-condition
        if (is_scalar($fn)) {
            $fn(42);
        }
    }
}

/** @param callable(int): string $fn */
function checkStringTwice(callable $fn): void
{
    if (is_string($fn)) {
        // @mago-expect analysis:redundant-type-comparison
        // @mago-expect analysis:redundant-condition
        if (is_string($fn)) {
            $fn(42);
        }
    }
}

/** @param callable(int): string $fn */
function checkScalarDirect(callable $fn): string
{
    if (is_scalar($fn)) {
        return $fn(42);
    }
    return $fn(42);
}

/** @param callable(int): string $fn */
function test_is_scalar_invalid_argument(callable $fn): void
{
    if (is_scalar($fn)) {
        // @mago-expect analysis:invalid-argument
        $fn('not-an-int');
    }
}

/** @param callable(int): string $fn */
function test_object_then_string(callable $fn): void
{
    if (is_object($fn)) {
        // @mago-expect analysis:impossible-type-comparison
        // @mago-expect analysis:impossible-condition
        if (is_string($fn)) {
        }
    }
}

/** @param callable(int): string $fn */
function test_string_then_object(callable $fn): void
{
    if (is_string($fn)) {
        // @mago-expect analysis:impossible-type-comparison
        // @mago-expect analysis:impossible-condition
        if (is_object($fn)) {
        }
    }
}
