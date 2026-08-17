<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Exception\InvalidArgumentException;

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
    ) {
        if (($literalKind === StringLiteralKind::Value) !== ($literalValue !== null)) {
            throw new InvalidArgumentException('Only a literal string type may carry a value, and it must carry one.');
        }
    }
}
