<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * The readable and writable contracts of an extension-provided magic property.
 *
 * A null side makes that operation invalid. At least one side must be present.
 *
 * @api
 */
final class PropertyType
{
    public function __construct(
        public readonly ?Type $readType = null,
        public readonly ?Type $writeType = null,
    ) {
        if ($readType === null && $writeType === null) {
            throw new InvalidArgumentException('An analyzer property type must be readable, writable, or both.');
        }
    }
}
