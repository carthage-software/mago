<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Syntax\NodeKind;

/**
 * Inspects selected syntax nodes after their containing file has been analyzed.
 *
 * Mago matches targets natively and batches only files containing matching
 * nodes. Hooks therefore receive final inferred types without scanning every
 * source file in PHP.
 *
 * @api
 * @extends TargetedAnalysisHook<NodeKind>
 */
interface NodeAnalysisHook extends TargetedAnalysisHook {}
