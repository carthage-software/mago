<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * @api
 */
final class ListElement
{
    public function __construct(
        public readonly int $index,
        public readonly bool $optional,
        public readonly Type $type,
    ) {
        if ($index < 0) {
            throw new InvalidArgumentException('A known list element index cannot be negative.');
        }
    }
}
