<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies flow-sensitive assertions for targeted method and static-method calls.
 *
 * @api
 * @extends TargetedProvider<MethodTarget>
 */
interface MethodAssertionProvider extends TargetedProvider
{
    public function getAssertions(AssertionProviderContext $context): ?InvocationAssertions;
}
