<?php

declare(strict_types=1);

/**
 * @template TParentFailure of Exception
 * @template TChildFailure of TParentFailure
 */
final class GenericThrowTemplateBoundChain
{
    /**
     * @param TChildFailure $failure
     * @throws TParentFailure
     */
    public function throwChildFailure(Exception $failure): void
    {
        throw $failure;
    }
}
