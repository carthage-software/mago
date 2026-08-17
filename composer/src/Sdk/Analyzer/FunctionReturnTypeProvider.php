<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies a more precise return type for targeted function calls.
 *
 * @api
 * @extends TargetedProvider<FunctionTarget>
 */
interface FunctionReturnTypeProvider extends TargetedProvider
{
    public function getReturnType(ReturnTypeProviderContext $context): ?Type;
}
