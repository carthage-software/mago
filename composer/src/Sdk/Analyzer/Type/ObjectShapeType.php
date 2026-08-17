<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class ObjectShapeType implements AtomicType
{
    /** @param list<ObjectProperty> $properties */
    public function __construct(
        public readonly array $properties,
        public readonly bool $sealed,
    ) {}

    public function __toString(): string
    {
        return 'object{...}';
    }
}
