<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
final class MixedType implements AtomicType
{
    public function __construct(
        public readonly bool $issetFromLoop,
        public readonly bool $nonNull,
        public readonly bool $empty,
        public readonly MixedTruthiness $truthiness,
    ) {}

    public function __toString(): string
    {
        return 'mixed';
    }
}
