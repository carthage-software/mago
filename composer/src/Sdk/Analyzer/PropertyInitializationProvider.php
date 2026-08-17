<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Marks targeted declared properties as initialized by external framework behavior.
 *
 * @api
 * @extends TargetedProvider<PropertyTarget>
 */
interface PropertyInitializationProvider extends TargetedProvider
{
    public function isPropertyInitialized(PropertyInitializationProviderContext $context): bool;
}
