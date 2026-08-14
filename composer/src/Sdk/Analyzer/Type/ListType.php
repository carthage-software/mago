<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * @api
 */
final class ListType implements AtomicType
{
    /** @param null|list<ListElement> $knownElements */
    public function __construct(
        public readonly Type $elementType,
        public readonly ?array $knownElements,
        public readonly ?int $knownCount,
        public readonly bool $nonEmpty,
    ) {
        if ($knownCount !== null && $knownCount < 0) {
            throw new InvalidArgumentException('A known list count cannot be negative.');
        }
    }

    public function __toString(): string
    {
        return 'list';
    }
}
