<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\SourceLocation;

/** @api */
final class AttributeMetadata
{
    public function __construct(
        public readonly string $name,
        public readonly SourceLocation $location,
    ) {}
}
