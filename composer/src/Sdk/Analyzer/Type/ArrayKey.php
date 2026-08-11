<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
final class ArrayKey
{
    public function __construct(
        public readonly ArrayKeyKind $kind,
        public readonly int|string|null $value,
        public readonly ?string $constant = null,
    ) {}
}
