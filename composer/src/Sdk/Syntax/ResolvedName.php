<?php

declare(strict_types=1);

namespace Mago\Sdk\Syntax;

use Mago\Sdk\Span;

/**
 * A name resolved by Mago within a syntax node.
 *
 * @api
 */
final class ResolvedName
{
    public function __construct(
        public readonly Span $span,
        public readonly string $name,
        public readonly bool $imported,
    ) {}
}
