<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies flow-sensitive assertions for targeted method and static-method calls.
 *
 * @api
 */
interface MethodAssertionProvider
{
    /** @return non-empty-list<MethodTarget> */
    public function getTargets(): array;

    public function getAssertions(AssertionProviderContext $context): ?InvocationAssertions;
}
