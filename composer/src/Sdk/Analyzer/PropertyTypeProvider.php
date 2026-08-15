<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Establishes targeted magic properties and supplies their read and write types.
 *
 * @api
 */
interface PropertyTypeProvider
{
    /** @return non-empty-list<PropertyTarget> */
    public function getTargets(): array;

    public function getPropertyType(PropertyTypeProviderContext $context): ?PropertyType;
}
