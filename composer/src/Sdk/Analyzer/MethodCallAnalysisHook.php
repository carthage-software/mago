<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Inspects semantically matched method calls after their file has been analyzed.
 *
 * Mago matches resolved receiver classes and method names natively. Only calls
 * matching at least one target cross the extension boundary.
 *
 * @api
 * @extends TargetedAnalysisHook<MethodTarget>
 */
interface MethodCallAnalysisHook extends TargetedAnalysisHook {}
