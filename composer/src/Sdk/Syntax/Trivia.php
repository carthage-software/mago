<?php

declare(strict_types=1);

namespace Mago\Sdk\Syntax;

use Mago\Sdk\Span;

/**
 * A source comment attached to the current syntax tree.
 *
 * @api
 */
final class Trivia
{
    public function __construct(
        public readonly TriviaKind $kind,
        public readonly Span $span,
    ) {}
}
