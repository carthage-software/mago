<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Linter;

use Mago\Sdk\Linter\Rule;
use Mago\Sdk\Linter\RuleDefinition;

/**
 * @internal
 */
final class RegisteredRule
{
    /**
     * @param int<0, 65535> $index
     */
    public function __construct(
        public readonly int $index,
        public readonly Rule $rule,
        public readonly RuleDefinition $definition,
    ) {}
}
