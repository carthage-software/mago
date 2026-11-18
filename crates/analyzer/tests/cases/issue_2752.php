<?php

declare(strict_types=1);

final class Result
{
    public string $value = '';
}

/**
 * @template-covariant T
 */
interface Promise
{
    /**
     * @template TFulfilled
     *
     * @param ?(callable((T is void ? null : T)): TFulfilled) $onFulfilled
     *
     * @return Promise<TFulfilled>
     */
    public function then(?callable $onFulfilled = null): Promise;

    /**
     * @param (T is void ? null : T) $value
     */
    public function resolve(mixed $value): void;

    /**
     * @return callable((T is void ? null : T)): void
     */
    public function listener(): callable;
}

/**
 * @param Promise<Result> $promise
 *
 * @return Promise<string>
 */
function untyped_closure(Promise $promise): Promise
{
    return $promise->then(static function ($result) {
        return $result->value;
    });
}

/**
 * @param Promise<Result> $promise
 *
 * @return Promise<string>
 */
function typed_closure(Promise $promise): Promise
{
    return $promise->then(static fn(Result $result): string => $result->value);
}

/**
 * @param Promise<void> $promise
 *
 * @return Promise<null>
 */
function void_promise(Promise $promise): Promise
{
    return $promise->then(static fn($value) => $value);
}

/**
 * @param Promise<Result> $promise
 */
function resolve_with_null(Promise $promise): void
{
    /** @mago-expect analysis:null-argument */
    $promise->resolve(null);
}

/**
 * @param Promise<Result> $promise
 */
function call_listener(Promise $promise): void
{
    $listener = $promise->listener();
    $listener(new Result());
    /** @mago-expect analysis:null-argument */
    $listener(null);
}
