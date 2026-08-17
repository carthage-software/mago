<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Linter;

use Mago\Sdk\Syntax\SourceFile;

/**
 * @internal
 */
final class LintRequest
{
    /**
     * @param list<int<0, 65535>> $activeRules
     */
    public function __construct(
        public readonly array $activeRules,
        public readonly SourceFile $file,
    ) {}
}
