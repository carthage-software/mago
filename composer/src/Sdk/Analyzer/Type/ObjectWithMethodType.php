<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class ObjectWithMethodType implements AtomicType
{
    /** @param null|list<AtomicType> $intersections */
    public function __construct(
        public readonly string $method,
        public readonly ?array $intersections,
    ) {}

    public function __toString(): string
    {
        return "has-method<'{$this->method}'>";
    }
}
