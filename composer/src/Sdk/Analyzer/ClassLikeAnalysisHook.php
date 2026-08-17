<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Inspects class-like declarations descending from selected ancestors.
 *
 * Mago resolves ancestry natively. Only descendant declarations matching at
 * least one target cross the extension boundary; the ancestors do not.
 *
 * @api
 * @extends TargetedAnalysisHook<ClassLikeTarget>
 */
interface ClassLikeAnalysisHook extends TargetedAnalysisHook {}
