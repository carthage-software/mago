<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies flow-sensitive assertions for targeted function calls.
 *
 * @api
 * @extends TargetedProvider<FunctionTarget>
 */
interface FunctionAssertionProvider extends TargetedProvider
{
    public function getAssertions(AssertionProviderContext $context): ?InvocationAssertions;
}
