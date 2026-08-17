<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies a more precise return type for targeted method or static-method calls.
 *
 * @api
 * @extends TargetedProvider<MethodTarget>
 */
interface MethodReturnTypeProvider extends TargetedProvider
{
    public function getReturnType(ReturnTypeProviderContext $context): ?Type;
}
