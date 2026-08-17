<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Runs once after all file results have been merged.
 *
 * @api
 */
interface AfterAnalysisHook
{
    public function afterAnalysis(AfterAnalysisContext $context): void;
}
