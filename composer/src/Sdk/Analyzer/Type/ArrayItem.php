<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class ArrayItem
{
    public function __construct(
        public readonly ArrayKey $key,
        public readonly bool $optional,
        public readonly Type $type,
    ) {}
}
