<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * An after-file hook registered for selected semantic or syntax targets.
 *
 * @template-covariant TTarget
 *
 * @api
 */
interface TargetedAnalysisHook
{
    /** @return non-empty-list<TTarget> */
    public function getTargets(): array;

    /** @return list<FileAnalysisRequirement> */
    public function getRequirements(): array;

    public function analyze(NodeAnalysisContext $context): void;
}
