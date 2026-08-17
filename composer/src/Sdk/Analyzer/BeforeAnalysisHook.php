<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Runs once before parallel file analysis begins.
 *
 * @api
 */
interface BeforeAnalysisHook
{
    public function beforeAnalysis(BeforeAnalysisContext $context): void;
}
