<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * @api
 */
final class KeyedArrayType implements AtomicType
{
    /** @param null|list<ArrayItem> $knownItems */
    public function __construct(
        public readonly ?array $knownItems,
        public readonly ?Type $keyType,
        public readonly ?Type $valueType,
        public readonly bool $nonEmpty,
    ) {
        if (($keyType === null) !== ($valueType === null)) {
            throw new InvalidArgumentException('A keyed array fallback requires both its key and value types.');
        }
    }

    public function __toString(): string
    {
        return 'array';
    }
}
