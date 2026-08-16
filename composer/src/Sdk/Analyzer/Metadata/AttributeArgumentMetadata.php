<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\SourceLocation;

/**
 * One positional or named constant argument supplied to an attribute.
 *
 * @api
 */
final class AttributeArgumentMetadata
{
    public function __construct(
        public readonly ?string $name,
        public readonly SourceLocation $location,
        public readonly ?SourceLocation $nameLocation,
        public readonly ?SourceLocation $valueLocation,
        public readonly ?Type $valueType,
    ) {}
}
