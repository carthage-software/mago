<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\PHPVersion;

/**
 * Context passed after one file has completed analysis.
 *
 * @api
 */
final class AfterFileAnalysisContext extends LifecycleContext
{
    public function __construct(
        PHPVersion $phpVersion,
        Codebase $codebase,
        TypeComparator $types,
        CancellationTokenInterface $cancellation,
        public readonly FileAnalysis $analysis,
    ) {
        parent::__construct($phpVersion, $codebase, $types, $cancellation);
    }
}
