<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\PHPVersion;

/**
 * Context passed to a before-analysis hook.
 *
 * @api
 */
final class BeforeAnalysisContext extends LifecycleContext
{
    public readonly ReferenceRegistry $references;

    public function __construct(
        PHPVersion $phpVersion,
        Codebase $codebase,
        TypeComparator $types,
        CancellationTokenInterface $cancellation,
    ) {
        parent::__construct($phpVersion, $codebase, $types, $cancellation);

        $this->references = new ReferenceRegistry();
    }
}
