<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;

/**
 * @internal
 */
final class ReportedIssue
{
    public function __construct(
        public readonly Level $level,
        public readonly string $code,
        public readonly Issue $issue,
    ) {}
}
