<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;
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

    /** @var array<string, VariableDefinedness>|null */
    private readonly ?array $variableDefinedness;

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
        $this->variableDefinedness = $data->variableDefinedness;
        parent::__construct($context->phpVersion, $context->codebase, $context->types, $context->cancellation);
    }

    /**
     * Returns whether a local variable exists immediately before the target node executes.
     *
     * The name may be passed with or without its leading `$`.
     */
    public function getVariableDefinedness(string $variable): ?VariableDefinedness
    {
        if ($variable === '' || $variable === '$') {
            throw new InvalidArgumentException('A variable name cannot be empty.');
        }

        $definedness = $this->variableDefinedness;
        if ($definedness === null) {
            return null;
        }

        if ($variable[0] !== '$') {
            $variable = '$' . $variable;
        }

        return $definedness[$variable] ?? VariableDefinedness::Undefined;
    }
}
