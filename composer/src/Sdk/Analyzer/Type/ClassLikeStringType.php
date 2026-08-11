<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class ClassLikeStringType
{
    public function __construct(
        public readonly ClassLikeStringVariant $variant,
        public readonly ?ClassLikeStringKind $kind = null,
        public readonly ?string $literal = null,
        public readonly ?string $parameterName = null,
        public readonly ?GenericParent $definingEntity = null,
        public readonly ?AtomicType $constraint = null,
    ) {}
}
