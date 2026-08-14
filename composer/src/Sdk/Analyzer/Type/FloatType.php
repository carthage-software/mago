<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * @api
 */
final class FloatType
{
    public function __construct(
        public readonly FloatTypeKind $kind,
        public readonly ?float $value = null,
    ) {
        if (($kind === FloatTypeKind::Literal) !== ($value !== null)) {
            throw new InvalidArgumentException('Only a literal float type may carry a value, and it must carry one.');
        }
    }
}
