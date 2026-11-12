<?php

declare(strict_types=1);

/**
 * @template TFailure of Exception
 */
final class GenericThrowUnusedDetection
{
    /**
     * @throws TFailure
     */
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
