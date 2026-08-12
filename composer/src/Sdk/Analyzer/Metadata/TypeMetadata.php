<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\SourceLocation;

/** @api */
final class TypeMetadata
{
    public function __construct(
        public readonly SourceLocation $location,
        public readonly Type $type,
        public readonly bool $fromDocblock,
        public readonly bool $inferred,
    ) {}
}
