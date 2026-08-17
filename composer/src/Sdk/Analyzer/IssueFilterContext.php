<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Reporting\ReportedIssue;

/**
 * Context for deciding whether one native analyzer issue should remain visible.
 *
 * The file contents are the exact in-memory bytes analyzed by Mago. They do not
 * need to match a file currently present on disk.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class IssueFilterContext
{
    public function __construct(
        public readonly PHPVersion $phpVersion,
        public readonly Codebase $codebase,
        public readonly TypeComparator $types,
        public readonly CancellationTokenInterface $cancellation,
        public readonly string $file,
        public readonly string $contents,
        public readonly ReportedIssue $issue,
    ) {}
}
