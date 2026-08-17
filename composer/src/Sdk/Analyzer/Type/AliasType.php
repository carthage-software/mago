<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class AliasType implements AtomicType
{
    public function __construct(
        public readonly string $class,
        public readonly string $alias,
    ) {}

    public function __toString(): string
    {
        return '!' . $this->class . '::' . $this->alias;
    }
}
