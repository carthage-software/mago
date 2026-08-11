<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
final class ResourceType implements AtomicType
{
    public function __construct(
        public readonly ?bool $closed,
    ) {}

    public function __toString(): string
    {
        return 'resource';
    }
}
