<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Establishes targeted magic properties and supplies their read and write types.
 *
 * @api
 * @extends TargetedProvider<PropertyTarget>
 */
interface PropertyTypeProvider extends TargetedProvider
{
    public function getPropertyType(PropertyTypeProviderContext $context): ?PropertyType;
}
