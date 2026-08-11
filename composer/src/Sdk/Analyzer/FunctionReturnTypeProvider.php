<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies a more precise return type for targeted function calls.
 *
 * @api
 */
interface FunctionReturnTypeProvider
{
    /** @return non-empty-list<FunctionTarget> */
    public function getTargets(): array;

    public function getReturnType(ReturnTypeProviderContext $context): ?Type;
}
