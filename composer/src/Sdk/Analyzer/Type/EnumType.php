<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class EnumType implements AtomicType
{
    public function __construct(
        public readonly string $name,
        public readonly ?string $case = null,
    ) {}

    public function __toString(): string
    {
        return $this->case === null ? $this->name : $this->name . '::' . $this->case;
    }
}
