<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Context passed to a before-analysis hook.
 *
 * @extends LifecycleContext<MutableCodebase>
 *
 * @api
 */
final class BeforeAnalysisContext extends LifecycleContext {}
