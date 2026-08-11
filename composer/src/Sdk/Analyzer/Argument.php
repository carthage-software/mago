<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Span;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class Argument
{
    public function __construct(
        public readonly ?string $name,
        public readonly bool $unpacked,
        public readonly bool $placeholder,
        public readonly Span $span,
        public readonly string $expression,
        public readonly ?Type $type,
    ) {}
}
