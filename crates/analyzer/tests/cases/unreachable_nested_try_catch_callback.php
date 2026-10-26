<?php

final class DeferredFailure extends Exception
{
}

interface AppThrowable extends Throwable
{
}

function check_unreachable_nested_try_catch(callable $dynamicCallable): void
{
    $callback = $dynamicCallable;
    $stored =
        /** @throws DeferredFailure */
        static function (mixed $value) use ($callback): void {
            $callback($value);
            throw new DeferredFailure();
        };

    try {
        try {
            $dynamicCallable();
            throw new LogicException();
        } catch (Exception) {
        } catch (RuntimeException) { // @mago-expect analysis:dead-catch
            array_map($stored, [1]);
        }
    } finally {
    }
}

function check_broad_throwable_error_catch(Throwable $throwable, callable $dynamicCallable): void
{
    try {
        try {
            throw $throwable;
        } catch (Exception) {
        } catch (Error) { // @mago-expect analysis:avoid-catching-error
            $dynamicCallable();
        } catch (Throwable) { // @mago-expect analysis:dead-catch
        }
    } catch (DeferredFailure) {
    } catch (Throwable) {
    }
}

function check_unresolved_catch_union(Throwable $throwable, callable $dynamicCallable): void
{
    try {
        try {
            $dynamicCallable();
        } catch (Exception) {
        } catch (MissingDeferredCatchType|RuntimeException) { // @mago-expect analysis:non-existent-catch-type
        } catch (Throwable) {
        }
    }
}

function check_inverse_known_catch_coverage(): void
{
    try {
        try {
            throw new RuntimeException();
        } catch (RuntimeException) {
        } catch (Exception) { // @mago-expect analysis:dead-catch
            throw new DeferredFailure();
        }
    } catch (DeferredFailure) { // @mago-expect analysis:dead-catch
    }
}

function check_cumulative_broad_throwable_catches(Throwable $throwable): void
{
    try {
        try {
            throw $throwable;
        } catch (Exception) {
        } catch (Error) { // @mago-expect analysis:avoid-catching-error
        } catch (Throwable) { // @mago-expect analysis:dead-catch
            throw new DeferredFailure();
        }
    } catch (DeferredFailure) { // @mago-expect analysis:dead-catch
    }
}

function check_union_broad_throwable_catch(Throwable $throwable): void
{
    try {
        try {
            throw $throwable;
        } catch (Exception|Error) { // @mago-expect analysis:avoid-catching-error
        } catch (Throwable) { // @mago-expect analysis:dead-catch
            throw new DeferredFailure();
        }
    } catch (DeferredFailure) { // @mago-expect analysis:dead-catch
    }
}

function check_subinterface_throwable_coverage(AppThrowable $throwable): void
{
    try {
        try {
            throw $throwable;
        } catch (Exception) {
        } catch (Error) { // @mago-expect analysis:avoid-catching-error
        } catch (AppThrowable) { // @mago-expect analysis:dead-catch
        }
    } catch (Throwable) { // @mago-expect analysis:dead-catch
    }
}

function check_unknown_condition_throw_with_known_branch_throw(callable $dynamicCallable): void
{
    if (true) { // @mago-expect analysis:redundant-condition
        try {
            try {
                if ($dynamicCallable()) {
                    throw new RuntimeException();
                }
            } catch (RuntimeException) {
            } catch (Error) { // @mago-expect analysis:avoid-catching-error
                throw new DeferredFailure();
            }
        } catch (DeferredFailure) {
        }
    }
}

function check_mixed_condition_throws(?bool $flag, callable $dynamicCallable): void
{
    try {
        try {
            if ((($flag ?? throw new RuntimeException()) === true) && ($dynamicCallable() === true)) {
            }
        } catch (RuntimeException) {
        } catch (Error) { // @mago-expect analysis:avoid-catching-error
            throw new DeferredFailure();
        }
    } catch (DeferredFailure) {
    }
}

function check_known_condition_throw(?bool $flag): void
{
    try {
        try {
            if ($flag ?? throw new RuntimeException()) {
            }
        } catch (RuntimeException) {
        } catch (LogicException) { // @mago-expect analysis:dead-catch
        }
    } catch (Throwable) { // @mago-expect analysis:dead-catch
    }
}

check_unreachable_nested_try_catch(static fn (): null => null);
