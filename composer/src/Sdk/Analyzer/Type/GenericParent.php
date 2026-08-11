<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/** @api */
final class GenericParent
{
    public function __construct(
        public readonly GenericParentKind $kind,
        public readonly string $name,
        public readonly ?string $member = null,
    ) {}
}
