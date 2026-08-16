<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Syntax\Node;
use Mago\Sdk\Syntax\SourceFile;

/**
 * Context passed for one targeted node after file analysis.
 *
 * @api
 */
final class NodeAnalysisContext extends LifecycleContext
{
    public readonly FileAnalysis $analysis;

    public function __construct(
        AfterFileAnalysisContext $context,
        public readonly SourceFile $source,
        public readonly Node $node,
    ) {
        $this->analysis = $context->analysis;
        parent::__construct($context->phpVersion, $context->codebase, $context->types, $context->cancellation);
    }
}
