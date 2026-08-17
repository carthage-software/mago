<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class SimpleAtomicType implements AtomicType
{
    public function __construct(
        public readonly SimpleAtomicTypeKind $kind,
    ) {}

    public function __toString(): string
    {
        return $this->kind->value;
    }
}
