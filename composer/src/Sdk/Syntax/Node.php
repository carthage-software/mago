<?php

declare(strict_types=1);

namespace Mago\Sdk\Syntax;

use Mago\Sdk\Span;

/**
 * One node in the filtered concrete syntax tree sent by Mago.
 *
 * @api
 */
final class Node
{
    /**
     * @param non-negative-int $id
     * @param null|non-negative-int $parentId
     */
    public function __construct(
        public readonly int $id,
        public readonly NodeKind $kind,
        public readonly Span $span,
        public readonly ?int $parentId,
    ) {}
}
