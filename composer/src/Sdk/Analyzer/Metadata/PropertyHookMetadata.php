<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\SourceLocation;

/**
 * Metadata for a PHP property get or set hook.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class PropertyHookMetadata
{
    /**
     * @param list<AttributeMetadata> $attributes
     */
    public function __construct(
        public readonly string $name,
        public readonly SourceLocation $location,
        public readonly MetadataFlags $flags,
        public readonly ?ParameterMetadata $parameter,
        public readonly bool $returnsByReference,
        public readonly bool $abstract,
        public readonly array $attributes,
        public readonly ?TypeMetadata $returnType,
        public readonly bool $hasDocblock,
    ) {}
}
