<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * File-analysis data a lifecycle hook wants embedded in its batch.
 *
 * @api
 */
enum FileAnalysisRequirement
{
    /** Embed every expression type in the file. Intended for after-file hooks. */
    case ExpressionTypes;

    /** Embed the inferred type of every node targeted by a node-analysis hook. */
    case TargetExpressionTypes;

    /** Embed the direct receiver type of targeted method and static-method calls. */
    case ReceiverType;

    /** Embed the direct argument value types of targeted calls. */
    case ArgumentTypes;

    /** Embed each targeted node's concrete-syntax subtree. */
    case TargetSubtree;

    /** Embed the exact source text of files containing targeted nodes. */
    case SourceText;
}
