<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Exception\InvalidArgumentException;

use function is_int;
use function is_string;

/**
 * @api
 */
final class ArrayKey
{
    public function __construct(
        public readonly ArrayKeyKind $kind,
        public readonly int|string|null $value,
        public readonly ?string $constant = null,
    ) {
        $valid = match ($kind) {
            ArrayKeyKind::Integer => is_int($value) && $constant === null,
            ArrayKeyKind::String => is_string($value) && $constant === null,
            ArrayKeyKind::ClassLikeConstant => is_string($value)
                && $value !== ''
                && $constant !== null
                && $constant !== '',
        };
        if (!$valid) {
            throw new InvalidArgumentException("Invalid value for array key kind `{$kind->name}`.");
        }
    }
}
