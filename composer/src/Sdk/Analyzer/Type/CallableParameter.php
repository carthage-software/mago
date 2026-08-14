<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class CallableParameter
{
    public function __construct(
        public readonly ?string $name,
        public readonly ?Type $type,
        public readonly bool $byReference,
        public readonly bool $variadic,
        public readonly bool $hasDefault,
    ) {}
}
