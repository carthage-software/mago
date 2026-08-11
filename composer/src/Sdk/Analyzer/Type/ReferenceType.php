<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class ReferenceType implements AtomicType
{
    /**
     * @param null|list<Type> $parameters
     * @param null|list<Variance> $variances
     * @param null|list<AtomicType> $intersections
     */
    public function __construct(
        public readonly ReferenceTypeKind $kind,
        public readonly ?string $name,
        public readonly ?array $parameters,
        public readonly ?array $variances,
        public readonly ?array $intersections,
        public readonly ?string $member,
        public readonly ?ReferenceSelectorKind $selector,
    ) {}

    public function __toString(): string
    {
        return $this->name ?? '*';
    }
}
