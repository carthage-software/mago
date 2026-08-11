<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
final class FunctionLikeIdentifier
{
    public function __construct(
        public readonly FunctionLikeKind $kind,
        public readonly string $name,
        public readonly ?string $class = null,
    ) {}
}
