<?php

declare(strict_types=1);

namespace Mago\Sdk;

/** A source span paired with its logical file name. @api */
final class SourceLocation
{
    public function __construct(
        public readonly ?string $file,
        public readonly Span $span,
    ) {}
}
