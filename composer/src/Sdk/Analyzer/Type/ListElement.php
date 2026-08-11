<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/** @api */
final class ListElement
{
    public function __construct(
        public readonly int $index,
        public readonly bool $optional,
        public readonly Type $type,
    ) {}
}
