<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies a more precise return type for targeted method or static-method calls.
 *
 * @api
 */
interface MethodReturnTypeProvider
{
    /** @return non-empty-list<MethodTarget> */
    public function getTargets(): array;

    public function getReturnType(ReturnTypeProviderContext $context): ?Type;
}
