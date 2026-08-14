<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;

/**
 * One directed edge in Mago's final symbol-reference graph.
 *
 * @api
 */
final class SymbolReference
{
    public function __construct(
        public readonly ReferenceOrigin $source,
        public readonly string|MemberIdentifier $target,
        public readonly ReferenceKind $kind,
    ) {}
}
