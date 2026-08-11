<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class StringType
{
    public function __construct(
        public readonly StringLiteralKind $literalKind,
        public readonly ?string $literalValue,
        public readonly bool $numeric,
        public readonly bool $truthy,
        public readonly bool $nonEmpty,
        public readonly bool $callable,
        public readonly StringCasing $casing,
    ) {}
}
