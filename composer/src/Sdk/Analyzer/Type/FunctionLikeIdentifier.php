<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * @api
 */
final class FunctionLikeIdentifier
{
    public function __construct(
        public readonly FunctionLikeKind $kind,
        public readonly string $name,
        public readonly ?string $class = null,
    ) {
        if ($name === '') {
            throw new InvalidArgumentException('A function-like identifier name cannot be empty.');
        }

        if ($kind === FunctionLikeKind::Method ? $class === null || $class === '' : $class !== null) {
            throw new InvalidArgumentException('Only a method identifier requires a non-empty class name.');
        }
    }
}
