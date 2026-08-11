<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/** @api */
final class KeyedArrayType implements AtomicType
{
    /** @param null|list<ArrayItem> $knownItems */
    public function __construct(
        public readonly ?array $knownItems,
        public readonly ?Type $keyType,
        public readonly ?Type $valueType,
        public readonly bool $nonEmpty,
    ) {}

    public function __toString(): string
    {
        return 'array';
    }
}
