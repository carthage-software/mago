<?php

declare(strict_types=1);

namespace Mago\Sdk\Syntax;

/**
 * One argument in a Mago call-expression view.
 *
 * @api
 */
final class CallArgument
{
    public function __construct(
        public readonly Node $node,
        public readonly Node $value,
        public readonly ?string $name,
        public readonly bool $unpacked,
    ) {}
}
