<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Exception\InvalidArgumentException;

use function count;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class NamedObjectType implements AtomicType
{
    /**
     * @param null|list<Type> $parameters
     * @param null|list<Variance> $variances
     * @param null|list<AtomicType> $intersections
     */
    public function __construct(
        public readonly string $name,
        public readonly ?array $parameters,
        public readonly ?array $variances,
        public readonly bool $static,
        public readonly bool $isThis,
        public readonly ?array $intersections,
        public readonly bool $remappedParameters,
    ) {
        if ($name === '') {
            throw new InvalidArgumentException('A named object type requires a non-empty class-like name.');
        }

        if ($parameters !== null && $variances !== null && count($parameters) !== count($variances)) {
            throw new InvalidArgumentException('Named object type parameters and variances must have equal lengths.');
        }
    }

    public function __toString(): string
    {
        return $this->name;
    }
}
