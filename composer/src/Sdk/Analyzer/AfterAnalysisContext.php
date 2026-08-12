<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\PHPVersion;

/**
 * Context passed after the whole analysis result has been merged.
 *
 * @extends LifecycleContext<Codebase>
 *
 * @api
 */
final class AfterAnalysisContext extends LifecycleContext
{
    public function __construct(
        PHPVersion $phpVersion,
        Codebase $codebase,
        TypeComparator $types,
        CancellationTokenInterface $cancellation,
        public readonly ProjectAnalysis $analysis,
    ) {
        parent::__construct($phpVersion, $codebase, $types, $cancellation);
    }
}
