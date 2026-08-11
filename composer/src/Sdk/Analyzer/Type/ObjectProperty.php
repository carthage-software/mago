<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/** @api */
final class ObjectProperty
{
    public function __construct(
        public readonly string $name,
        public readonly bool $optional,
        public readonly Type $type,
    ) {}
}
