<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/** @api */
final class ConditionalType implements AtomicType
{
    public function __construct(
        public readonly Type $subject,
        public readonly Type $target,
        public readonly Type $then,
        public readonly Type $otherwise,
        public readonly bool $negated,
    ) {}

    public function __toString(): string
    {
        return 'conditional-type';
    }
}
