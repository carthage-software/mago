<?php

declare(strict_types=1);

/**
 * @template TFailure of Exception
 */
final class GenericThrowingContract
{
    /** @param TFailure $failure */
    public function __construct(Exception $failure)
    {
    }

    /** @throws TFailure */
    public function noThrow(): void
    {
    }

    /**
     * @param TFailure $failure
     * @throws TFailure
     */
    public function throwFailure(Exception $failure): void
    {
        throw $failure;
    }
}

interface FailureMarker
{
}

/**
 * @template TFailure of Exception&FailureMarker
 */
final class GenericIntersectionThrowingContract
{
    /** @throws TFailure */
    public function noThrow(): void
    {
    }

    /**
     * @param TFailure $failure
     * @throws TFailure
     */
    public function throwFailure(Exception $failure): void
    {
        throw $failure;
    }

    /** @throws TFailure */
    public function throwUnrelatedException(): void
    {
        throw new Exception();
    }
}

/**
 * @template TFailure of Exception&Throwable
 */
final class GenericThrowableIntersectionCatch
{
    /**
     * @param TFailure $failure
     */
    public function catchException(Exception $failure): void
    {
        try {
            throw $failure;
        } catch (Exception) {
        }
    }
}

/**
 * @template TFailure of RuntimeException|LogicException
 */
final class GenericUnionCatch
{
    /**
     * @param TFailure $failure
     * @throws TFailure
     */
    public function catchBoth(Throwable $failure): void
    {
        try {
            throw $failure;
        } catch (RuntimeException) {
        } catch (LogicException) {
        }
    }
}

/**
 * @template TFailure of RuntimeException|LogicException
 */
final class GenericPartialUnionCatch
{
    /** @param TFailure $failure */
    public function catchRuntimeException(Throwable $failure): void
    {
        try {
            // @mago-expect analysis:unhandled-thrown-type
            throw $failure;
        } catch (RuntimeException) {
        }
    }
}

/**
 * @template TFailure of Throwable
 */
final class GenericThrowableUnionCatch
{
    /**
     * @param TFailure $failure
     * @throws TFailure
     */
    public function catchThrowable(Throwable $failure): void
    {
        try {
            throw $failure;
        } catch (Exception|Error) {
        }
    }

    /** @param TFailure $failure */
    public function catchThrowableInSiblingClauses(Throwable $failure): void
    {
        try {
            throw $failure;
        } catch (Exception) {
        } catch (Error) {
        }
    }
}

/**
 * @template TFailure of Exception
 * @param TFailure $failure
 * @throws TFailure
 */
function throwGenericFailure(Exception $failure): void
{
    throw $failure;
}

/**
 * @template TFailure of Exception
 * @param TFailure $failure
 */
function invokeGenericFailure(Exception $failure): void
{
    // @mago-expect analysis:unhandled-thrown-type
    throwGenericFailure($failure);
}

/**
 * @template TFailure of Exception
 * @param TFailure $failure
 * @throws TFailure
 */
function rethrowGenericFailure(Exception $failure): void
{
    throwGenericFailure($failure);
}

/**
 * @template TFailure of Exception
 * @param TFailure $failure
 */
function catchGenericInvocation(Exception $failure): void
{
    try {
        throwGenericFailure($failure);
    } catch (Exception) {
    }
}

/**
 * @template TFirstFailure of Exception
 * @template TSecondFailure of Exception
 */
final class GenericTemplateIdentityContract
{
    /**
     * @param TSecondFailure $failure
     * @throws TFirstFailure
     */
    public function throwWrongTemplate(Exception $failure): void
    {
        throw $failure;
    }
}
