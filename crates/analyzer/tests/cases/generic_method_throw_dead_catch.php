<?php

declare(strict_types=1);

/**
 * @template TFailure of Exception
 */
final class GenericMethodThrower
{
    /**
     * @param TFailure $failure
     * @throws TFailure
     */
    public function throwFailure(Exception $failure): void
    {
        throw $failure;
    }
}

/**
 * @template TFailure of Exception
 * @param GenericMethodThrower<TFailure> $thrower
 * @param TFailure $failure
 * @throws TFailure
 */
function generic_method_throw_dead_catch(GenericMethodThrower $thrower, Exception $failure): void
{
    try {
        $thrower->throwFailure($failure);
    // @mago-expect analysis:dead-catch,analysis:avoid-catching-error
    } catch (Error) {
    }
}

/**
 * @template TFailure of Exception
 * @param GenericMethodThrower<TFailure> $thrower
 * @param TFailure $failure
 */
function generic_method_throw_caught(GenericMethodThrower $thrower, Exception $failure): void
{
    try {
        $thrower->throwFailure($failure);
    } catch (Exception) {
    }
}
