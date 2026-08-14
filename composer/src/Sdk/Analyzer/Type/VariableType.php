<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class VariableType implements AtomicType
{
    public function __construct(
        public readonly string $name,
    ) {}

    public function __toString(): string
    {
        return $this->name;
    }
}
