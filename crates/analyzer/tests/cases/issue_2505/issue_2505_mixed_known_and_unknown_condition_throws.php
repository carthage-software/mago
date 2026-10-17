<?php

final class MixedConditionDeferredFailure extends Exception
{
}

/**
 * @throws RuntimeException
 */
function condition_with_known_throw(?bool $flag): bool
{
    return $flag ?? throw new RuntimeException();
}

/** @param callable(): bool $dynamicCallable */
function catch_known_and_unknown_condition_throws(?bool $flag, callable $dynamicCallable): void
{
    try {
        try {
            if (condition_with_known_throw($flag) && $dynamicCallable()) {
            }
        } catch (RuntimeException) {
        } catch (Error) { // @mago-expect analysis:avoid-catching-error
            throw new MixedConditionDeferredFailure();
        }
    } catch (MixedConditionDeferredFailure) {
    }
}
