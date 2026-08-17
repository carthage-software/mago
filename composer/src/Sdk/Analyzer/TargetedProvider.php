<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * A provider registered for one or more analyzer targets.
 *
 * @template-covariant TTarget
 *
 * @api
 */
interface TargetedProvider
{
    /** @return non-empty-list<TTarget> */
    public function getTargets(): array;
}
