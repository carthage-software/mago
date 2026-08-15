<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Reporting\ReportedIssue;

/**
 * @internal
 */
final class IssueFilterRequest
{
    /**
     * @param list<int<0, 65535>> $hookIndices
     * @param non-empty-string $file
     * @param list<ReportedIssue> $issues
     */
    public function __construct(
        public readonly int $generation,
        public readonly array $hookIndices,
        public readonly string $file,
        public readonly string $contents,
        public readonly array $issues,
    ) {}
}
