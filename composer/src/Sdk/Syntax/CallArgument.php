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
    /**
     * @param non-negative-int $index Source-order index in the containing call.
     */
    public function __construct(
        public readonly int $index,
        public readonly Node $node,
        public readonly Node $value,
        public readonly ?string $name,
        public readonly bool $unpacked,
    ) {}
}
