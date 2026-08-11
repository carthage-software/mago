<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
final class ScalarType implements AtomicType
{
    public function __construct(
        public readonly ScalarTypeKind $kind,
        public readonly bool|IntegerType|FloatType|StringType|ClassLikeStringType|null $refinement = null,
    ) {}

    public function __toString(): string
    {
        return $this->kind->value;
    }
}
