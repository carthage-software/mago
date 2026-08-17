<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Aggregate dependency-reference counts produced by analysis.
 *
 * @api
 */
final class ReferenceSummary
{
    public function __construct(
        public readonly int $body,
        public readonly int $signature,
        public readonly int $maps,
    ) {}
}
