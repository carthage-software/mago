<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Internal\Analyzer\NodeAnalysisData;
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

    /** References shared by every hook targeting this file. */
    public readonly ReferenceRegistry $references;

    /** The target's inferred type when TargetExpressionTypes was requested. */
    public readonly ?Type $targetType;

    /** The direct call receiver's type when ReceiverType was requested. */
    public readonly ?Type $receiverType;

    /**
     * Direct argument value types in source order when ArgumentTypes was requested.
     *
     * @var list<Type|null>
     */
    public readonly array $argumentTypes;

    public function __construct(
        AfterFileAnalysisContext $context,
        public readonly SourceFile $source,
        public readonly Node $node,
        NodeAnalysisData $data,
    ) {
        $this->analysis = $context->analysis;
        $this->references = $context->references;
        $this->targetType = $data->targetType;
        $this->receiverType = $data->receiverType;
        $this->argumentTypes = $data->argumentTypes;
        parent::__construct($context->phpVersion, $context->codebase, $context->types, $context->cancellation);
    }
}
