<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
final class FloatType
{
    public function __construct(
        public readonly FloatTypeKind $kind,
        public readonly ?float $value = null,
    ) {}
}
