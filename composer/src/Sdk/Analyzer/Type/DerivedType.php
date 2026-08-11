<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/** @api */
final class DerivedType implements AtomicType
{
    /**
     * Operands preserve the order defined by the corresponding derived type.
     *
     * @param list<Type> $operands
     * @param list<AtomicType> $intersections
     */
    public function __construct(
        public readonly DerivedTypeKind $kind,
        public readonly array $operands,
        public readonly array $intersections = [],
        public readonly ?Visibility $visibility = null,
    ) {}

    public function __toString(): string
    {
        return $this->kind->name;
    }
}
