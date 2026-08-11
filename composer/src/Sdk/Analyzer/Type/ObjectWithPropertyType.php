<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
final class ObjectWithPropertyType implements AtomicType
{
    /** @param null|list<AtomicType> $intersections */
    public function __construct(
        public readonly string $property,
        public readonly ?array $intersections,
    ) {}

    public function __toString(): string
    {
        return "has-property<'{$this->property}'>";
    }
}
