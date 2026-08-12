<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\Analyzer\ReportedIssue;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;

/**
 * Common read-only state and diagnostic reporting for analyzer lifecycle hooks.
 *
 * @template-covariant TCodebase of Codebase
 *
 * @api
 */
abstract class LifecycleContext
{
    /**
     * @var list<ReportedIssue>
     */
    private array $issues = [];

    /** @param TCodebase $codebase */
    public function __construct(
        public readonly PHPVersion $phpVersion,
        public readonly Codebase $codebase,
        public readonly TypeComparator $types,
        public readonly CancellationTokenInterface $cancellation,
    ) {}

    public function report(Level $level, string $code, Issue $issue): void
    {
        if ($code === '') {
            throw new InvalidArgumentException('An analyzer hook issue code cannot be empty.');
        }

        $this->issues[] = new ReportedIssue($level, $code, $issue);
    }

    /**
     * @internal
     *
     * @return list<ReportedIssue>
     */
    final public function takeReportedIssues(): array
    {
        $issues = $this->issues;
        $this->issues = [];

        return $issues;
    }
}
