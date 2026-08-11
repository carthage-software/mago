<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
final class IntegerType
{
    public function __construct(
        public readonly IntegerTypeKind $kind,
        public readonly ?int $minimum = null,
        public readonly ?int $maximum = null,
    ) {}

    public function getLiteralValue(): ?int
    {
        return $this->kind === IntegerTypeKind::Literal ? $this->minimum : null;
    }
}
