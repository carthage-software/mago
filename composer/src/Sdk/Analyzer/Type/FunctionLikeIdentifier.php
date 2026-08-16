<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Exception\InvalidArgumentException;

use function strcasecmp;

/**
 * @api
 * @mago-expect lint:cyclomatic-complexity
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

    public function equals(self $other): bool
    {
        if ($this->kind !== $other->kind) {
            return false;
        }

        if ($this->kind === FunctionLikeKind::Closure) {
            return $this->name === $other->name;
        }

        return strcasecmp($this->name, $other->name) === 0
            && ($this->class === null || strcasecmp($this->class, $other->class ?? '') === 0);
    }
}
