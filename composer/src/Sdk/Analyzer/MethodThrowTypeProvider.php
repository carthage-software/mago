<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies the exception class names thrown by targeted method calls.
 *
 * Return `null` to fall through to another provider or the native contract.
 * Return an empty list when the method call does not throw.
 *
 * @api
 * @extends TargetedProvider<MethodTarget>
 */
interface MethodThrowTypeProvider extends TargetedProvider
{
    /** @return list<non-empty-string>|null */
    public function getThrownExceptions(ThrowTypeProviderContext $context): ?array;
}
