<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Analyzer\Type;

/** @api */
final class CallableConstraint
{
    /**
     * @param list<string> $parameterNames
     */
    public function __construct(
        public readonly array $parameterNames,
        public readonly Type $inputType,
        public readonly Type $parameterType,
    ) {}
}
