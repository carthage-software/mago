<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies flow-sensitive assertions for targeted function calls.
 *
 * @api
 */
interface FunctionAssertionProvider
{
    /** @return non-empty-list<FunctionTarget> */
    public function getTargets(): array;

    public function getAssertions(AssertionProviderContext $context): ?InvocationAssertions;
}
