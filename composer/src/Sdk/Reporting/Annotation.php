<?php

declare(strict_types=1);

namespace Mago\Sdk\Reporting;

use Mago\Sdk\Span;

/**
 * A source range highlighted by a reported issue.
 *
 * @api
 */
final class Annotation
{
    public function __construct(
        public readonly AnnotationKind $kind,
        public readonly Span $span,
        public readonly ?string $message = null,
    ) {}
}
