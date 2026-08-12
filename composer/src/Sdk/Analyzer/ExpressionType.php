<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Span;

/**
 * An inferred type paired with its expression range.
 *
 * @api
 */
final class ExpressionType
{
    public function __construct(
        public readonly Span $span,
        public readonly Type $type,
    ) {}
}
