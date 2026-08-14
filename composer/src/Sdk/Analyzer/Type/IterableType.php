<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class IterableType implements AtomicType
{
    /** @param null|list<AtomicType> $intersections */
    public function __construct(
        public readonly Type $keyType,
        public readonly Type $valueType,
        public readonly ?array $intersections,
    ) {}

    public function __toString(): string
    {
        return 'iterable';
    }
}
