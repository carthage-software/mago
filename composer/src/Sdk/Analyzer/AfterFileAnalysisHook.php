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
    /** @return list<FileAnalysisRequirement> */
    public function getRequirements(): array;

    public function afterFileAnalysis(AfterFileAnalysisContext $context): void;
}
