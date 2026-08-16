<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * File-analysis data an after-file hook wants embedded in its lifecycle batch.
 *
 * @api
 */
enum FileAnalysisRequirement
{
    case ExpressionTypes;
}
