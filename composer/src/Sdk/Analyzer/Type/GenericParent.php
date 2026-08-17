<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * @api
 */
final class GenericParent
{
    public function __construct(
        public readonly GenericParentKind $kind,
        public readonly string $name,
        public readonly ?string $member = null,
    ) {
        if ($kind === GenericParentKind::ClassLike && $name === '') {
            throw new InvalidArgumentException('A class-like generic parent name cannot be empty.');
        }

        if ($kind === GenericParentKind::FunctionLike ? $member === null : $member !== null) {
            throw new InvalidArgumentException('Only a function-like generic parent requires a member component.');
        }

        if ($member === '') {
            throw new InvalidArgumentException('A function-like generic parent member cannot be empty.');
        }
    }
}
