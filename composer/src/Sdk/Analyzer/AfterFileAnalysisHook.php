<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Runs after each analyzed file completes.
 *
 * @api
 */
interface AfterFileAnalysisHook
{
    public function afterFileAnalysis(AfterFileAnalysisContext $context): void;
}
