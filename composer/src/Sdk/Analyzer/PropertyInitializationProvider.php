<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Marks targeted declared properties as initialized by external framework behavior.
 *
 * @api
 */
interface PropertyInitializationProvider
{
    /** @return non-empty-list<PropertyTarget> */
    public function getTargets(): array;

    public function isPropertyInitialized(PropertyInitializationProviderContext $context): bool;
}
