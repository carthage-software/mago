<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum ReferenceSelectorKind
{
    case Wildcard;
    case Identifier;
    case StartsWith;
    case EndsWith;
}
