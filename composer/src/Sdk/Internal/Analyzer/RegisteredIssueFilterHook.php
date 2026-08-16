<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\IssueFilterHook;

/**
 * @internal
 */
final class RegisteredIssueFilterHook
{
    /**
     * @param non-empty-string $plugin
     * @param non-empty-list<non-empty-string> $codes
     */
    public function __construct(
        public readonly int $index,
        public readonly string $plugin,
        public readonly IssueFilterHook $hook,
        public readonly array $codes,
    ) {}
}
