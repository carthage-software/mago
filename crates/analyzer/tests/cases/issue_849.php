<?php

declare(strict_types=1);

/**
 * Test that non-empty-mixed is correctly parsed as a valid type
 *
 * @param non-empty-mixed $value
 * @return non-empty-mixed
 */
function accept_truthy(mixed $value): mixed
{
    return accept_truthy($value);
}

/**
 * Test union with non-empty-mixed
 *
 * @param non-empty-mixed|null $value
 * @return non-empty-mixed|null
 */
function accept_truthy_or_null(mixed $value): mixed
{
    return accept_truthy_or_null($value);
}

/**
 * Test non-empty-mixed in conditional return type
 *
 * @template T
 * @param T $value
 * @return (T is non-empty-mixed ? T : null)
 */
function truthy_or_null(mixed $value): mixed
{
    return truthy_or_null($value);
}

/**
 * @template TValue
 * @template TParams of mixed
 * @template TException of Throwable
 * @template TExceptionValue of TException|class-string<TException>|string
 *
 * @param  TValue $condition
 * @param  Closure(TParams): TExceptionValue|TExceptionValue $exception
 * @param  TParams ...$parameters
 * @return TValue
 *
 * @throws TException
 */
function throw_if(
    mixed $condition,
    Closure|Throwable|string $exception = RuntimeException::class,
    mixed ...$parameters,
): mixed {
    // @mago-expect analysis:unhandled-thrown-type
    return throw_if($condition, $exception, ...$parameters);
}

/**
 * @throws RuntimeException
 */
function example(): void
{
    accept_truthy(1);
    accept_truthy('hello');
    accept_truthy([1, 2, 3]);
    accept_truthy(new stdClass());
    accept_truthy(true);
    accept_truthy_or_null(null);
    accept_truthy_or_null(1);
    throw_if(false, RuntimeException::class, message: 'some message');
    throw_if(false, new RuntimeException());
    throw_if(false, fn(): RuntimeException => new RuntimeException());
}

throw_if(false, 'some message');
