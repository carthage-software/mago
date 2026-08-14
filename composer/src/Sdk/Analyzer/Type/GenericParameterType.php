<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class GenericParameterType implements AtomicType
{
    /** @param null|list<AtomicType> $intersections */
    public function __construct(
        public readonly string $name,
        public readonly Type $constraint,
        public readonly GenericParent $definingEntity,
        public readonly ?array $intersections,
    ) {}

    public function __toString(): string
    {
        return $this->name;
    }
}
